//! LTV (Long-Term Validation) data cleanup.
//!
//! PDF documents with digital signatures may embed LTV data in the
//! Document Security Store (DSS) dictionary. This includes:
//! - CRLs (Certificate Revocation Lists)
//! - OCSP responses (Online Certificate Status Protocol)
//! - Certificates (X.509 certificate chain)
//! - VRI (Validation Related Information) per-signature entries
//!
//! Cleaning up LTV data reduces file size and removes stale validation
//! data after signatures are stripped.

use crate::error::{PdfError, Result};
use crate::pdf_reader::PdfReader;
use lopdf::{Dictionary, Object, ObjectId};

/// Information about LTV data embedded in a PDF.
#[derive(Debug, Clone)]
pub struct LtvInfo {
    /// Number of certificates found in the DSS.
    pub cert_count: usize,
    /// Number of CRLs found in the DSS.
    pub crl_count: usize,
    /// Number of OCSP responses found in the DSS.
    pub ocsp_count: usize,
    /// Number of VRI (per-signature validation) entries.
    pub vri_count: usize,
    /// Estimated total size in bytes of all LTV data.
    pub estimated_size: usize,
    /// Certificate subjects (if parseable).
    pub cert_subjects: Vec<String>,
}

/// Result of an LTV cleanup operation.
#[derive(Debug, Clone)]
pub struct LtvCleanupResult {
    /// Number of objects removed from the document.
    pub objects_removed: usize,
    /// Estimated bytes saved.
    pub bytes_saved: usize,
    /// Whether the DSS dictionary was removed entirely.
    pub dss_removed: bool,
}

/// Detect LTV data in a PDF's Document Security Store (DSS).
///
/// Returns detailed information about embedded validation data.
pub fn detect_ltv(reader: &PdfReader) -> Result<Option<LtvInfo>> {
    let doc = reader.document();

    let catalog = doc
        .trailer
        .get_deref(b"Root", doc)
        .and_then(|o| o.as_dict())
        .map_err(|_| PdfError::XfaPacketNotFound("no Root catalog".to_string()))?;

    let dss = match catalog.get_deref(b"DSS", doc) {
        Ok(Object::Dictionary(d)) => d,
        _ => return Ok(None),
    };

    let certs = get_stream_array(dss, b"Certs", doc);
    let crls = get_stream_array(dss, b"CRLs", doc);
    let ocsps = get_stream_array(dss, b"OCSPs", doc);

    let vri_count = match dss.get_deref(b"VRI", doc) {
        Ok(Object::Dictionary(d)) => d.len(),
        _ => 0,
    };

    // Estimate total size
    let mut estimated_size = 0;
    for (_, data) in &certs {
        estimated_size += data.len();
    }
    for (_, data) in &crls {
        estimated_size += data.len();
    }
    for (_, data) in &ocsps {
        estimated_size += data.len();
    }

    // Extract certificate subjects (basic DER parsing)
    let cert_subjects: Vec<String> = certs
        .iter()
        .filter_map(|(_, data)| extract_cert_subject(data))
        .collect();

    Ok(Some(LtvInfo {
        cert_count: certs.len(),
        crl_count: crls.len(),
        ocsp_count: ocsps.len(),
        vri_count,
        estimated_size,
        cert_subjects,
    }))
}

/// Remove all LTV data from a PDF.
///
/// Removes the entire DSS dictionary and all referenced stream objects.
/// Returns information about what was cleaned up.
pub fn remove_ltv(reader: &mut PdfReader) -> Result<LtvCleanupResult> {
    let info = detect_ltv(reader)?;
    let info = match info {
        Some(i) => i,
        None => {
            return Ok(LtvCleanupResult {
                objects_removed: 0,
                bytes_saved: 0,
                dss_removed: false,
            })
        }
    };

    let doc = reader.document();

    let catalog_ref = doc
        .trailer
        .get(b"Root")
        .and_then(|o| o.as_reference())
        .map_err(|_| PdfError::XfaPacketNotFound("no Root in trailer".to_string()))?;

    let catalog = doc
        .get_object(catalog_ref)
        .and_then(|o| o.as_dict())
        .map_err(|_| PdfError::XfaPacketNotFound("Root not a dictionary".to_string()))?
        .clone();

    // Collect all DSS-related object IDs for removal
    let dss_ref = match catalog.get(b"DSS") {
        Ok(Object::Reference(r)) => Some(*r),
        _ => None,
    };

    let dss_dict = match catalog.get(b"DSS") {
        Ok(Object::Reference(r)) => {
            let r = *r;
            match doc.get_object(r) {
                Ok(Object::Dictionary(d)) => Some(d.clone()),
                _ => None,
            }
        }
        Ok(Object::Dictionary(d)) => Some(d.clone()),
        _ => None,
    };

    let mut objects_to_remove: Vec<ObjectId> = Vec::new();

    if let Some(dss) = &dss_dict {
        // Collect stream references from Certs, CRLs, OCSPs
        for key in &[b"Certs".as_ref(), b"CRLs".as_ref(), b"OCSPs".as_ref()] {
            if let Ok(Object::Array(arr)) = dss.get(key) {
                for item in arr {
                    if let Object::Reference(r) = item {
                        objects_to_remove.push(*r);
                    }
                }
            }
        }

        // Collect VRI entries and their sub-objects
        if let Ok(Object::Reference(vri_ref)) = dss.get(b"VRI") {
            let vri_ref = *vri_ref;
            if let Ok(Object::Dictionary(vri)) = doc.get_object(vri_ref) {
                for (_, value) in vri.iter() {
                    if let Object::Reference(r) = value {
                        // Each VRI entry is a dictionary with Cert/CRL/OCSP arrays
                        if let Ok(Object::Dictionary(entry)) = doc.get_object(*r) {
                            for sub_key in &[b"Cert".as_ref(), b"CRL".as_ref(), b"OCSP".as_ref()] {
                                if let Ok(Object::Array(arr)) = entry.get(sub_key) {
                                    for item in arr {
                                        if let Object::Reference(sub_r) = item {
                                            objects_to_remove.push(*sub_r);
                                        }
                                    }
                                }
                            }
                            objects_to_remove.push(*r);
                        }
                    }
                }
                objects_to_remove.push(vri_ref);
            }
        } else if let Ok(Object::Dictionary(vri)) = dss.get(b"VRI") {
            // Inline VRI dictionary
            for (_, value) in vri.iter() {
                if let Object::Reference(r) = value {
                    objects_to_remove.push(*r);
                }
            }
        }
    }

    // Add the DSS object itself
    if let Some(dss_id) = dss_ref {
        objects_to_remove.push(dss_id);
    }

    // --- Begin mutations ---
    let doc = reader.document_mut();

    // Remove DSS from catalog
    if let Ok(Object::Dictionary(ref mut cat)) = doc.get_object_mut(catalog_ref) {
        cat.remove(b"DSS");
    }

    // Remove all collected objects
    let mut removed = 0;
    for obj_id in &objects_to_remove {
        if doc.objects.remove(obj_id).is_some() {
            removed += 1;
        }
    }

    Ok(LtvCleanupResult {
        objects_removed: removed,
        bytes_saved: info.estimated_size,
        dss_removed: true,
    })
}

/// Remove only specific VRI entries (by signature hash key).
///
/// This allows selective cleanup of validation data for removed signatures
/// while preserving LTV data for remaining valid signatures.
pub fn remove_vri_entries(reader: &mut PdfReader, hash_keys: &[&str]) -> Result<usize> {
    let doc = reader.document();

    let catalog_ref = doc
        .trailer
        .get(b"Root")
        .and_then(|o| o.as_reference())
        .map_err(|_| PdfError::XfaPacketNotFound("no Root in trailer".to_string()))?;

    let catalog = doc
        .get_object(catalog_ref)
        .and_then(|o| o.as_dict())
        .map_err(|_| PdfError::XfaPacketNotFound("Root not a dictionary".to_string()))?
        .clone();

    let dss_ref = match catalog.get(b"DSS") {
        Ok(Object::Reference(r)) => *r,
        _ => return Ok(0),
    };

    let dss = match doc.get_object(dss_ref) {
        Ok(Object::Dictionary(d)) => d.clone(),
        _ => return Ok(0),
    };

    let vri_ref = match dss.get(b"VRI") {
        Ok(Object::Reference(r)) => *r,
        _ => return Ok(0),
    };

    let vri = match doc.get_object(vri_ref) {
        Ok(Object::Dictionary(d)) => d.clone(),
        _ => return Ok(0),
    };

    // Collect VRI entry objects to remove
    let mut vri_objects_to_remove: Vec<ObjectId> = Vec::new();
    let keys_to_remove: Vec<Vec<u8>> = hash_keys.iter().map(|k| k.as_bytes().to_vec()).collect();

    for key in &keys_to_remove {
        if let Ok(Object::Reference(r)) = vri.get(key) {
            // Collect sub-objects from this VRI entry
            if let Ok(Object::Dictionary(entry)) = doc.get_object(*r) {
                for sub_key in &[b"Cert".as_ref(), b"CRL".as_ref(), b"OCSP".as_ref()] {
                    if let Ok(Object::Array(arr)) = entry.get(sub_key) {
                        for item in arr {
                            if let Object::Reference(sub_r) = item {
                                vri_objects_to_remove.push(*sub_r);
                            }
                        }
                    }
                }
            }
            vri_objects_to_remove.push(*r);
        }
    }

    // --- Begin mutations ---
    let doc = reader.document_mut();

    // Remove keys from VRI dictionary
    if let Ok(Object::Dictionary(ref mut vri_dict)) = doc.get_object_mut(vri_ref) {
        for key in &keys_to_remove {
            vri_dict.remove(key);
        }
    }

    // Remove collected objects
    let mut removed = 0;
    for obj_id in &vri_objects_to_remove {
        if doc.objects.remove(obj_id).is_some() {
            removed += 1;
        }
    }

    Ok(removed)
}

/// Get an array of stream data from a DSS dictionary entry.
fn get_stream_array(
    dss: &Dictionary,
    key: &[u8],
    doc: &lopdf::Document,
) -> Vec<(ObjectId, Vec<u8>)> {
    let arr = match dss.get(key) {
        Ok(Object::Array(a)) => a,
        _ => return vec![],
    };

    arr.iter()
        .filter_map(|item| {
            if let Object::Reference(r) = item {
                if let Ok(Object::Stream(s)) = doc.get_object(*r) {
                    if let Ok(data) = s.get_plain_content() {
                        return Some((*r, data));
                    }
                }
            }
            None
        })
        .collect()
}

/// Extract a basic certificate subject from DER-encoded X.509 data.
///
/// This does minimal ASN.1 parsing — enough to find the CN (Common Name)
/// in the Subject field. Not a full X.509 parser.
fn extract_cert_subject(der_data: &[u8]) -> Option<String> {
    // Look for common CN OID (2.5.4.3 = 55 04 03) followed by UTF8String or PrintableString
    let cn_oid = [0x55, 0x04, 0x03];

    for i in 0..der_data.len().saturating_sub(cn_oid.len() + 4) {
        if der_data[i..].starts_with(&cn_oid) {
            // Skip OID, then read the string value
            let offset = i + cn_oid.len();
            if offset + 2 > der_data.len() {
                continue;
            }

            let tag = der_data[offset];
            let len = der_data[offset + 1] as usize;

            // UTF8String (0x0C), PrintableString (0x13), IA5String (0x16)
            if matches!(tag, 0x0C | 0x13 | 0x16) && offset + 2 + len <= der_data.len() {
                let value = &der_data[offset + 2..offset + 2 + len];
                if let Ok(s) = std::str::from_utf8(value) {
                    return Some(s.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Document, Object, Stream};

    /// Build a PDF with a DSS dictionary containing LTV data.
    fn build_pdf_with_ltv(cert_count: usize, crl_count: usize, ocsp_count: usize) -> Vec<u8> {
        let mut doc = Document::with_version("1.7");

        // Create certificate streams
        let mut cert_refs = Vec::new();
        for i in 0..cert_count {
            // Minimal DER-like data with a CN OID
            let mut cert_data = vec![0x30, 0x20]; // SEQUENCE
            cert_data.extend_from_slice(&[0x55, 0x04, 0x03]); // CN OID
            cert_data.push(0x13); // PrintableString
            let cn = format!("Test Cert {i}");
            cert_data.push(cn.len() as u8);
            cert_data.extend_from_slice(cn.as_bytes());
            cert_data.extend_from_slice(&[0x00; 50]); // padding

            let stream = Stream::new(Dictionary::new(), cert_data);
            let id = doc.add_object(Object::Stream(stream));
            cert_refs.push(Object::Reference(id));
        }

        // Create CRL streams
        let mut crl_refs = Vec::new();
        for _ in 0..crl_count {
            let stream = Stream::new(Dictionary::new(), vec![0x30; 100]);
            let id = doc.add_object(Object::Stream(stream));
            crl_refs.push(Object::Reference(id));
        }

        // Create OCSP streams
        let mut ocsp_refs = Vec::new();
        for _ in 0..ocsp_count {
            let stream = Stream::new(Dictionary::new(), vec![0x30; 50]);
            let id = doc.add_object(Object::Stream(stream));
            ocsp_refs.push(Object::Reference(id));
        }

        // VRI dictionary
        let vri_entry = dictionary! {
            "Cert" => Object::Array(cert_refs.clone()),
        };
        let vri_entry_id = doc.add_object(Object::Dictionary(vri_entry));

        let vri = dictionary! {
            "ABC123" => Object::Reference(vri_entry_id),
        };
        let vri_id = doc.add_object(Object::Dictionary(vri));

        // DSS dictionary
        let dss = dictionary! {
            "Certs" => Object::Array(cert_refs),
            "CRLs" => Object::Array(crl_refs),
            "OCSPs" => Object::Array(ocsp_refs),
            "VRI" => Object::Reference(vri_id),
        };
        let dss_id = doc.add_object(Object::Dictionary(dss));

        // Pages
        let pages = dictionary! {
            "Type" => Object::Name(b"Pages".to_vec()),
            "Kids" => Object::Array(vec![]),
            "Count" => Object::Integer(0),
        };
        let pages_id = doc.add_object(Object::Dictionary(pages));

        // Catalog with DSS
        let catalog = dictionary! {
            "Type" => Object::Name(b"Catalog".to_vec()),
            "Pages" => Object::Reference(pages_id),
            "DSS" => Object::Reference(dss_id),
        };
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        buf
    }

    fn build_pdf_without_dss() -> Vec<u8> {
        let mut doc = Document::with_version("1.4");
        let pages = dictionary! {
            "Type" => Object::Name(b"Pages".to_vec()),
            "Kids" => Object::Array(vec![]),
            "Count" => Object::Integer(0),
        };
        let pages_id = doc.add_object(Object::Dictionary(pages));
        let catalog = dictionary! {
            "Type" => Object::Name(b"Catalog".to_vec()),
            "Pages" => Object::Reference(pages_id),
        };
        let catalog_id = doc.add_object(Object::Dictionary(catalog));
        doc.trailer.set("Root", Object::Reference(catalog_id));

        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        buf
    }

    #[test]
    fn detect_ltv_finds_data() {
        let pdf = build_pdf_with_ltv(3, 2, 1);
        let reader = PdfReader::from_bytes(&pdf).unwrap();
        let info = detect_ltv(&reader).unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.cert_count, 3);
        assert_eq!(info.crl_count, 2);
        assert_eq!(info.ocsp_count, 1);
        assert_eq!(info.vri_count, 1);
        assert!(info.estimated_size > 0);
    }

    #[test]
    fn detect_ltv_parses_cert_subjects() {
        let pdf = build_pdf_with_ltv(2, 0, 0);
        let reader = PdfReader::from_bytes(&pdf).unwrap();
        let info = detect_ltv(&reader).unwrap().unwrap();
        assert_eq!(info.cert_subjects.len(), 2);
        assert_eq!(info.cert_subjects[0], "Test Cert 0");
        assert_eq!(info.cert_subjects[1], "Test Cert 1");
    }

    #[test]
    fn detect_ltv_returns_none_without_dss() {
        let pdf = build_pdf_without_dss();
        let reader = PdfReader::from_bytes(&pdf).unwrap();
        assert!(detect_ltv(&reader).unwrap().is_none());
    }

    #[test]
    fn remove_ltv_cleans_up() {
        let pdf = build_pdf_with_ltv(2, 1, 1);
        let mut reader = PdfReader::from_bytes(&pdf).unwrap();

        // LTV exists before
        assert!(detect_ltv(&reader).unwrap().is_some());

        let result = remove_ltv(&mut reader).unwrap();
        assert!(result.dss_removed);
        assert!(result.objects_removed > 0);
        assert!(result.bytes_saved > 0);

        // Verify DSS is gone after save/reload
        let saved = reader.save_to_bytes().unwrap();
        let reader2 = PdfReader::from_bytes(&saved).unwrap();
        assert!(detect_ltv(&reader2).unwrap().is_none());
    }

    #[test]
    fn remove_ltv_noop_without_dss() {
        let pdf = build_pdf_without_dss();
        let mut reader = PdfReader::from_bytes(&pdf).unwrap();
        let result = remove_ltv(&mut reader).unwrap();
        assert_eq!(result.objects_removed, 0);
        assert!(!result.dss_removed);
    }

    #[test]
    fn remove_ltv_reduces_file_size() {
        let pdf = build_pdf_with_ltv(5, 3, 2);
        let original_size = pdf.len();

        let mut reader = PdfReader::from_bytes(&pdf).unwrap();
        remove_ltv(&mut reader).unwrap();

        let saved = reader.save_to_bytes().unwrap();
        assert!(
            saved.len() < original_size,
            "cleaned PDF ({}) should be smaller than original ({})",
            saved.len(),
            original_size
        );
    }

    #[test]
    fn remove_vri_selective() {
        let pdf = build_pdf_with_ltv(1, 0, 0);
        let mut reader = PdfReader::from_bytes(&pdf).unwrap();

        let removed = remove_vri_entries(&mut reader, &["ABC123"]).unwrap();
        assert!(removed > 0);

        // DSS should still exist (only VRI entry removed)
        let saved = reader.save_to_bytes().unwrap();
        let reader2 = PdfReader::from_bytes(&saved).unwrap();
        let info = detect_ltv(&reader2).unwrap();
        // DSS still present but VRI should be empty
        assert!(info.is_some());
        assert_eq!(info.unwrap().vri_count, 0);
    }

    #[test]
    fn remove_vri_nonexistent_key_returns_zero() {
        let pdf = build_pdf_with_ltv(1, 0, 0);
        let mut reader = PdfReader::from_bytes(&pdf).unwrap();
        let removed = remove_vri_entries(&mut reader, &["NONEXISTENT"]).unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn remove_ltv_pdf_remains_valid() {
        let pdf = build_pdf_with_ltv(3, 2, 1);
        let mut reader = PdfReader::from_bytes(&pdf).unwrap();
        remove_ltv(&mut reader).unwrap();

        let saved = reader.save_to_bytes().unwrap();
        let reader2 = PdfReader::from_bytes(&saved).unwrap();
        // Should still be loadable
        assert!(reader2.document().trailer.get(b"Root").is_ok());
    }

    #[test]
    fn extract_cert_subject_parses_cn() {
        // DER-like data with CN OID followed by PrintableString
        let mut data = vec![0x30, 0x20]; // SEQUENCE header
        data.extend_from_slice(&[0x55, 0x04, 0x03]); // CN OID
        data.push(0x13); // PrintableString
        data.push(11); // length
        data.extend_from_slice(b"Test CA Inc");

        let subject = extract_cert_subject(&data);
        assert_eq!(subject.as_deref(), Some("Test CA Inc"));
    }
}
