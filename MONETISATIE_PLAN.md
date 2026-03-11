# Monetisatieplan XFA SDK & PDFluent

> **Versie:** 2.0 — 11 maart 2026
> **Status:** Definitief strategisch plan
> **Model:** Closed-source, gratis persoonlijk gebruik, betaald zakelijk (JetBrains-model)
> **Gerelateerd:** [LICENSING_AND_ENTERPRISE_DEPLOYMENT.md](../PDFluent/LICENSING_AND_ENTERPRISE_DEPLOYMENT.md) — technische licentie-implementatie

---

## Inhoudsopgave

1. [Executive Summary](#1-executive-summary)
2. [Uitgangspositie](#2-uitgangspositie)
3. [Strategisch Model: "Layered Moat"](#3-strategisch-model-layered-moat)
4. [Licentiestructuur](#4-licentiestructuur)
5. [Betalingsinfrastructuur](#5-betalingsinfrastructuur)
6. [Prijsstrategie](#6-prijsstrategie)
7. [Go-to-Market per laag](#7-go-to-market-per-laag)
8. [Tijdlijn: Maand-voor-maand acties](#8-tijdlijn-maand-voor-maand-acties)
9. [Reseller & Distributie Strategie](#9-reseller--distributie-strategie)
10. [Financiële Projecties](#10-financiële-projecties)
11. [Exit-opties & Waardering](#11-exit-opties--waardering)
12. [Risico's & Mitigatie](#12-risicos--mitigatie)
13. [Concurrentiepositie & USPs](#13-concurrentiepositie--usps)
14. [KPI's & Meetbaarheid](#14-kpis--meetbaarheid)
15. [Beslissingslog](#15-beslissingslog)

---

## 1. Executive Summary

### Het plan in drie zinnen

PDFluent wordt een gratis, closed-source desktop PDF editor die als credibility-motor dient voor de commerciële XFA SDK. Iedereen krijgt dezelfde volledige versie — zakelijk gebruik vereist een betaalde licentie (JetBrains-model). Omzet komt uit drie gestapelde lagen: PDFluent Business licenties, SDK self-serve verkoop (per-product model), en enterprise deals via resellers.

### Kernbeslissingen

| Beslissing | Keuze | Rationale |
|------------|-------|-----------|
| Editor licentie | Closed source, gratis persoonlijk | Maximale adoptie zonder open-source overhead |
| SDK licentie | Closed source, per-product pricing | Industry-standaard, simpeler voor kopers dan per-developer |
| Feature-gating editor | Geen — iedereen krijgt alles | Simpelheid, goodwill, brede adoptie |
| Betalingen self-serve | LemonSqueezy (MoR) | Btw, compliance, invoicing tot ~€10K |
| Betalingen enterprise | Directe facturatie + bankoverschrijving | PO-support, NET-30, IBAN — wat enterprise verwacht |
| Licentie-technologie | Ed25519 signed license files | Offline, geen externe dienst, zelfde systeem voor editor + SDK |
| SDK evaluatie | 30-dagen tijdslimiet, geen watermark | Pragmatisch: volledige SDK, geen frictie, grote bedrijven betalen |
| Open source | Nee (noch editor, noch SDK) | Minder overhead, betere bescherming |

### Verwachte resultaten

| Metric | Maand 6 | Maand 12 | Maand 24 |
|--------|---------|----------|----------|
| PDFluent downloads | 10.000 | 50.000 | 200.000 |
| PDFluent Business licenties | 30 | 200 | 800 |
| SDK klanten | 0 | 5 | 15-25 |
| MRR totaal | €1.500 | €12.000 | €50.000+ |
| ARR | €18.000 | €144.000 | €600.000+ |

---

## 2. Uitgangspositie

### Wat we hebben

- **XFA Native Rust SDK**: 38 crates, 100% pure Rust, 449K+ PDFs zonder crashes
- **PDFluent**: Desktop PDF editor (Tauri v2), privacy-first
- **Bewezen kwaliteit**: 99,9964% pass rate op 9.509 PDFs, 0 crashes
- **Technische USPs**: XFA 3.3, FormCalc, WASM, PDF/A, PDF/UA, ZUGFeRD, digitale handtekeningen, OCR
- **Bindings**: C API, Node.js, Python, WASM

### Wat we missen

- Bereik: 0 gebruikers, geen community, geen merk
- Team: solo founder
- Omzet: €0
- Website/documentatie: nog niet live

### Marktcontext

- PDF SDK markt: $0,23B (2024) → $0,48B (2033), CAGR 8,68%
- Desktop PDF editor markt: $4,7B
- Adobe Acrobat: $240/jaar, breed gehaat
- Apryse koopt agressief: iText, PDFlib, Qoppa, LEAD Technologies, Eversign
- Anti-SaaS sentiment groeit
- Geen volwaardige gratis desktop PDF editor zonder beperkingen

---

## 3. Strategisch Model: "Layered Moat"

### Drie omzetlagen die onafhankelijk werken

```
┌─────────────────────────────────────────────────────────┐
│  LAAG 3: Enterprise SDK + Resellers (€15K-€200K/deal)   │  ← maand 9+
├─────────────────────────────────────────────────────────┤
│  LAAG 2: SDK Self-Serve + Cloud API (€1.990-€5.900/jr)  │  ← maand 4+
├─────────────────────────────────────────────────────────┤
│  LAAG 1: PDFluent Business Licenties (€99/user/jr)      │  ← maand 1+
├─────────────────────────────────────────────────────────┤
│  FUNDAMENT: PDFluent Personal (gratis, bereik & bewijs)  │  ← maand 1+
└─────────────────────────────────────────────────────────┘
```

### Waarom drie lagen?

| Laag | Functie | Revenue type | Risico als het wegvalt |
|------|---------|-------------|----------------------|
| Fundament | Bereik, credibility, PR | Geen (investering) | Geen directe revenue-impact |
| Laag 1 | Voorspelbare basisinkomsten | Recurring (SaaS-achtig) | Laag per klant, volume nodig |
| Laag 2 | Schaalbare developer revenue | Recurring + usage | Medium, self-serve |
| Laag 3 | High-value enterprise deals | Recurring, groot | Hoog per klant, maar weinig klanten |

**Het punt**: als één laag tegenvalt, compenseren de andere twee. Een pure enterprise-strategie is fragiel. Een pure consumer-strategie heeft lage marges. De combinatie is robuust.

### Verwachte revenue-mix bij volwassenheid

| Bron | Aandeel |
|------|---------|
| Editor licenties | ~30% |
| SDK self-serve | ~20% |
| Enterprise deals | ~50% |

Het meeste revenue komt uiteindelijk van een klein aantal enterprise klanten. De editor en self-serve SDK vormen de funnel daarheen.

---

## 4. Licentiestructuur

### PDFluent (Desktop Editor)

**Model: JetBrains-stijl — één versie, zelf-declaratie**

```
PDFluent
├── Persoonlijk gebruik     → Gratis (geen licentie nodig)
├── Educatie & non-profit   → Gratis (geen licentie nodig)
└── Zakelijk gebruik        → Licentie vereist (€99/user/jaar)
```

**Kerneigenschappen:**
- **Eén enkele build** — iedereen downloadt exact dezelfde app
- **Geen feature-verschil** — gratis gebruikers hebben toegang tot alles
- **Zelf-declaratie** — bij eerste start kiest de gebruiker "Persoonlijk" of "Zakelijk"
- **Zakelijk = licentie-activatie** — via signed license file
- **Persoonlijk = gewoon gebruiken** — geen registratie, geen beperkingen

**Wat telt als "zakelijk gebruik"?**
Zakelijk gebruik is elk gebruik in de context van een bedrijf, overheidsinstelling, of organisatie met meer dan 1 medewerker, ongeacht of het bedrijf winst maakt. Dit omvat:
- Gebruik door een werknemer voor werkgerelateerde taken
- Gebruik door een freelancer voor klantwerk
- Gebruik binnen een overheidsorganisatie

**Wat is "persoonlijk gebruik"?**
- Privégebruik thuis (eigen belastingaangifte, persoonlijke documenten)
- Educatief gebruik (studenten, docenten)
- Non-profit organisaties (optioneel — kan ook gratis)

**Technische implementatie in Tauri:**
```
┌──────────────────────────────────┐
│  Eerste keer opstarten           │
│                                  │
│  Hoe gebruikt u PDFluent?        │
│                                  │
│  [Persoonlijk]  [Zakelijk]       │
│                                  │
│  Persoonlijk: gratis, direct     │
│  Zakelijk: voer licentie in     │
└──────────────────────────────────┘
```

- Keuze wordt opgeslagen in lokale config
- "Zakelijk" toont invoerveld voor license key of license file
- Validatie via Ed25519 signature verificatie (offline, geen API call)
- Geen harde blokkade — app werkt altijd, maar toont periodiek herinnering bij zakelijk gebruik zonder key
- Gebruiker kan altijd switchen via Instellingen

> **Technische details:** Zie [LICENSING_AND_ENTERPRISE_DEPLOYMENT.md](../PDFluent/LICENSING_AND_ENTERPRISE_DEPLOYMENT.md) voor het volledige Ed25519 signing schema, licentiebestand format, en enterprise deployment.

### XFA SDK (Developer Library)

**Model: Per-product licentie — gecompileerde distributie**

```
XFA SDK
├── Evaluation     → Gratis (30 dagen, volledige SDK, geen watermark)
├── Startup        → €1.990/product/jaar (≤3 developers)
├── Business       → €5.900/product/jaar (≤10 developers)
├── Enterprise     → Vanaf €15.000/jaar (ongelimiteerd, SLA)
└── OEM            → Vanaf €25.000/jaar (redistributierechten)
```

**Waarom per-product en niet per-developer?**

Bedrijven kopen geen developer seats — ze kopen het recht om software te bouwen met de SDK. Dit is de dominante trend in de PDF SDK-industrie:
- Simpler voor de koper: geen "developers tellen"
- Natuurlijke schaling: meer producten = upgrade
- Beter voor procurement: past bij budgettering per project/product
- Hogere deal sizes: een product-licentie is makkelijker te verantwoorden dan N× developer seats

**SDK Evaluatie — pragmatische aanpak:**

De SDK wordt gecompileerd gedistribueerd (geen broncode). De evaluatieperiode werkt als volgt:
- **Volledige functionaliteit** — geen watermarks, geen rate limits, geen feature-beperkingen
- **30-dagen tijdslimiet** — na 30 dagen vanaf eerste gebruik retourneert de SDK een duidelijke foutmelding
- **Tijdstempel in binary** — elke maand een nieuwe evaluatie-build op de website
- **Geen online validatie vereist** — alles werkt offline

Na de evaluatieperiode heeft de developer twee opties:
1. Download een nieuwe evaluatie-build (reset naar 30 dagen)
2. Koop een licentie (license file, onbeperkt gebruik)

**Bewuste keuze: geen watermark op SDK-output.** De SDK wordt ook gebruikt voor het lezen van PDFs, niet alleen genereren. Een watermark zou de evaluatie-ervaring verslechteren. Het risico op piraterij is acceptabel — grote organisaties betalen uit compliance-overwegingen, ongeacht technische enforcement.

**SDK Licentie-management via Ed25519 signed license files:**

Zelfde technologie als de editor (zie [LICENSING_AND_ENTERPRISE_DEPLOYMENT.md](../PDFluent/LICENSING_AND_ENTERPRISE_DEPLOYMENT.md)):
- Signed JSON licentiebestand met product-rechten, expiry, en developer-limiet
- Public key ingebakken in de compiled SDK
- Initialisatie: `XfaSdk::init_with_license("path/to/license.json")?`
- Geen externe dienst, geen online checks, volledig offline

**SDK Redistributieregels:**

De licentie staat toe:
- SDK inbedden in software waar PDF-verwerking een *feature* is (boekhoudpakketten, HR-tools, rapportage-platforms)

De licentie verbiedt:
- Bouwen van general-purpose PDF editors
- Bouwen van concurrerende PDF SDKs
- Bouwen van document processing platforms die concurreren met PDFluent of de XFA SDK

Dit is standaard in de industrie (Apryse, iText, Nutrient hanteren dezelfde restrictie).

---

## 5. Betalingsinfrastructuur

### Architectuur — twee kanalen

```
                    SELF-SERVE (tot ~€10K)              ENTERPRISE (€10K+)
                    ─────────────────────               ──────────────────
Klant koopt         Via website/checkout                Via email/sales
        │                   │                                   │
        ▼                   ▼                                   ▼
┌──────────────────┐  LemonSqueezy (MoR)         Directe facturatie
│  PDFluent (1-9)  │  Handelt af: betaling,      (Moneybird / handmatig)
│  SDK self-serve  │  btw, invoicing, refunds    + bankoverschrijving (IBAN)
└────────┬─────────┘  135+ landen                + PO-support, NET-30
         │                                               │
         ▼                                               ▼
   Ed25519 license file                          Ed25519 license file
   (automatisch of handmatig)                    (handmatig gegenereerd)
```

### Kanaal 1: Self-serve via LemonSqueezy (dag 1)

**Geschikt voor:**
- PDFluent Business licenties (1-9 seats)
- SDK Startup en Business tiers
- Alles tot ~€10K/jaar

**Wat LemonSqueezy doet:**
| Aspect | Detail |
|--------|--------|
| Commissie | 5% + $0,50 per transactie |
| Btw-afhandeling | Ja (135+ landen, incl. EU btw-reverse charge) |
| Betaalmethoden | Creditcard, PayPal, Apple Pay, Google Pay, iDEAL (via Stripe) |
| Invoicing | Automatische facturen (PDF) voor klanten |
| License keys | Ingebouwde license key generatie |
| Refunds | Geautomatiseerd |

**Wat LemonSqueezy NIET kan (en waarom kanaal 2 nodig is):**
- Geen purchase order (PO) support
- Geen NET-30/60 betalingstermijnen
- Geen bankoverschrijving als betaalmethode voor kopers
- Niet geschikt voor enterprise procurement workflows

### Kanaal 2: Directe facturatie + bankoverschrijving (maand 1+)

**Geschikt voor:**
- PDFluent Business volume (10+ seats)
- SDK Enterprise en OEM
- Alle deals boven ~€10K/jaar
- Klanten die PO/NET-30/bankovermaking vereisen

**Flow:**
```
1. Klant neemt contact op (email, website formulier)
2. Offerte/quote sturen (PDF)
3. Klant stuurt Purchase Order (PO)
4. Factuur sturen tegen de PO (via Moneybird of handmatig)
5. Klant betaalt via IBAN-bankoverschrijving (NET-30)
6. License file genereren en leveren
```

**Facturatie-tools (kies één):**
| Tool | Prijs | Geschikt voor |
|------|-------|---------------|
| Moneybird | €11/maand | Nederlands, populair bij ZZP/MKB, btw-aangifte |
| Exact Online | €30+/maand | Nederlands, enterprise-grade, boekhouding |
| FreshBooks | €15/maand | Internationaal, mooi design |
| Handmatige PDF | €0 | Eerste deals, simpelst mogelijk |

**Aanbeveling:** Start met Moneybird. Nederlands, betaalbaar, genereert automatisch btw-aangifte, en stuurt professionele facturen met IBAN-gegevens.

### Waarom GEEN Keygen meer?

Het oorspronkelijke plan gebruikte Keygen.sh voor SDK license management. Dit is geschrapt om drie redenen:

1. **Onnodige complexiteit** — Ed25519 signed license files zijn simpeler en doen hetzelfde
2. **Onnodige kosten** — Keygen kost €99/maand bij groei; Ed25519 signing is gratis
3. **Consistentie** — editor én SDK gebruiken nu dezelfde licentie-technologie
4. **Offline-first** — geen externe API dependency voor licentievalidatie

De Ed25519 approach biedt alles wat nodig is:
- Cryptografisch veilige validatie (onmogelijk te vervalsen zonder private key)
- Volledig offline (geen internet vereist)
- Seats, expiry, features, en product-rechten in het licentiebestand
- Enterprise deployment: IT kopieert bestand naar centrale locatie
- Eén signing tool genereert licenties voor zowel editor als SDK

### Flow: Klant koopt PDFluent Business (1-9 seats)

```
1. Klant klikt "Zakelijke licentie kopen" in app of op website
2. → LemonSqueezy checkout pagina
3. → Klant betaalt (creditcard, iDEAL, PayPal)
4. → LemonSqueezy genereert license key
5. → Webhook triggert license file generatie (of: handmatig)
6. → Klant ontvangt license file per email
7. → Klant plaatst file in app of voert key in
8. → App valideert Ed25519 signature (offline)
9. → Licentie actief, herinnering verdwijnt
```

### Flow: Klant koopt PDFluent Business (10+ seats)

```
1. Klant contacteert via website of email
2. → Offerte met volumekorting (€79/seat/jaar bij 10+)
3. → Klant stuurt PO
4. → Factuur via Moneybird met IBAN
5. → Betaling via bankoverschrijving (NET-30)
6. → License file genereren (seats=N, customer=naam)
7. → IT distribueert license file + app centraal
```

### Flow: Klant koopt SDK licentie (self-serve)

```
1. Developer kiest tier op xfa-sdk.com
2. → LemonSqueezy checkout (Startup of Business)
3. → Bij betaling: license file wordt gegenereerd
4. → Developer ontvangt license file per email
5. → Plaatst file naast SDK of in config directory
6. → SDK valideert Ed25519 signature bij init
7. → Evaluatielimiet verdwijnt, productie-klaar
```

### Flow: Klant koopt SDK Enterprise/OEM

```
1. Klant neemt contact op via xfa-sdk.com/enterprise
2. → Discovery call: requirements, volume, SLA-behoefte
3. → Custom offerte
4. → PO + contract ondertekening
5. → Factuur + betaling via bankoverschrijving
6. → License file met enterprise-rechten
7. → Optioneel: dedicated Slack channel voor support
```

---

## 6. Prijsstrategie

### PDFluent Pricing

| Gebruik | Prijs | Wat de klant krijgt |
|---------|-------|---------------------|
| Persoonlijk | **Gratis** | Volledige app, geen beperkingen |
| Zakelijk (1-9 users) | **€99/user/jaar** | Volledige app + license file + email support |
| Zakelijk (10-49 users) | **€79/user/jaar** | Volume korting + prioriteit support |
| Zakelijk (50+ users) | **Contact** | Custom pricing + onboarding + enterprise deployment |

**Pricing rationale:**
- Adobe Acrobat Pro: €240/jaar → PDFluent is 60% goedkoper
- PDF Expert: €80/jaar → PDFluent is vergelijkbaar
- PDFgear: gratis maar closed source, beperkte features
- €99/jaar is een "no-brainer" voor bedrijven — lager dan één uur IT-consultancy

**Betaalkanaal:**
- 1-9 seats: LemonSqueezy (self-serve, creditcard/iDEAL)
- 10+ seats: Directe facturatie + bankoverschrijving (IBAN)

### XFA SDK Pricing

| Tier | Prijs | Doelgroep | Wat je krijgt |
|------|-------|-----------|---------------|
| **Evaluation** | Gratis | Iedereen | 30 dagen volledige SDK, geen watermark |
| **Startup** | €1.990/product/jaar | Startups (<€500K omzet) | 1 product, ≤3 developers, productie-gebruik |
| **Business** | €5.900/product/jaar | Mid-market, ISVs | 1 product, ≤10 developers, priority support |
| **Enterprise** | Vanaf €15.000/jaar | Grote organisaties | Ongelimiteerd, meerdere producten, SLA, security reviews |
| **OEM** | Vanaf €25.000/jaar | Software vendors | Redistributierechten, white-label, extended legal terms |

**Pricing rationale en marktpositionering:**
| Vendor | Typische prijs | Model |
|--------|---------------|-------|
| IronPDF | ~$2.000 eenmalig | Per-developer, perpetual |
| Nutrient/PSPDFKit | ~€5.000/user/jaar | Per-user, subscription |
| Apryse | $15K-$210K+/jaar | Per-seat + deployment |
| iText | $10K-$210K+/jaar | Per-PDF-volume |
| **XFA SDK** | **€1.990-€15K+/jaar** | **Per-product, subscription** |

Onze positie: **midden van de markt** met een aantrekkelijke instap (€1.990) en duidelijk pad naar enterprise (€15K+). Goedkoper dan Apryse/iText, vergelijkbaar met Nutrient, maar met unieke XFA-differentiatie.

**Publiceer pricing op de website.** Transparantie bouwt vertrouwen. Enterprise en OEM = "contact us" voor custom deals.

### Cloud API Pricing (toekomstig, fase 2+)

| Volume | Prijs per document |
|--------|--------------------|
| 0-500 docs/maand | Gratis |
| 501-5.000 | €0,05/doc |
| 5.001-50.000 | €0,02/doc |
| 50.000+ | Custom |

---

## 7. Go-to-Market per laag

### FUNDAMENT: PDFluent Personal (gratis, bereik)

**Doel**: Zoveel mogelijk downloads genereren als basis voor credibility en lead generation.

**Positionering:**
> "De gratis Adobe Acrobat alternatief — 3MB, privacy-first, jouw data blijft van jou."

Dit is geen open source verhaal. Het is een **gratis + privacy + kwaliteit** verhaal.

**Distributiekanalen:**

| Kanaal | Prioriteit | Verwacht bereik |
|--------|-----------|-----------------|
| pdfluent.com (direct download) | Hoog | Hoofdkanaal |
| Mac App Store | Hoog | Organisch bereik |
| Microsoft Store | Hoog | Windows gebruikers |
| Product Hunt | Hoog | Launch-moment |
| Hacker News | Hoog | Developer community |
| AlternativeTo.net | Medium | SEO + vergelijking |
| Reddit (r/macapps, r/privacy, r/windows) | Medium | Niche communities |
| Softpedia, MajorGeeks, Ninite | Medium | Download portals |
| YouTube (tech reviewers) | Medium | Visuele demo |

**PR-angles (kies per kanaal):**
1. **Anti-Adobe**: "3MB vs 7GB. Gratis vs €240/jaar. Jouw data vs hun cloud."
2. **Privacy-first**: "Geen cloud, geen account, geen telemetrie. Alles lokaal."
3. **Solo developer vs megacorp**: "Eén developer bouwt wat Adobe met duizenden doet."
4. **Technisch**: "Gebouwd in Rust — memory-safe, razendsnel, 3MB."

### LAAG 1: PDFluent Business (€99/user/jaar)

**Doel**: Stabiele recurring revenue van bedrijven die PDFluent al gratis gebruiken.

**Conversie-strategie:**
PDFluent Business is geen "upgrade" met extra features. Het is dezelfde app met een zakelijke licentie. De conversie komt door:

1. **Compliance-druk**: Bedrijven willen legaal software gebruiken
2. **Periodieke herinnering**: Na 30 dagen zakelijk gebruik zonder license → subtiele banner "Zakelijk gebruik? Activeer uw licentie."
3. **Support**: Business licentie = email support, personal = community/zelf
4. **Factuurnodig**: Bedrijven hebben een factuur nodig voor de boekhouding → LemonSqueezy (self-serve) of Moneybird (enterprise) levert automatisch

**Verwachte conversieratio:**
- Van alle downloads: ~5% is zakelijk gebruik
- Van zakelijk gebruik: ~10-15% koopt licentie (compliance-gedreven)
- Netto conversie: ~0,5-0,75% van totale downloads

**Target accounts:**
- MKB bedrijven die nu Adobe Acrobat betalen (€240/user/jaar → besparing van €141/user)
- Overheidsinstellingen met budgetdruk
- Non-profits en scholen (gratis of korting)
- Freelancers en ZZP'ers (€99/jaar is aftrekbaar)

### LAAG 2: SDK Self-Serve (€1.990-€5.900/product/jaar)

**Doel**: Developers vinden, evalueren, en kopen de SDK zonder menselijke interactie.

**Developer experience flow:**
```
1. Developer zoekt "rust pdf sdk" of "xfa pdf processing"
2. → Vindt xfa-sdk.com of crates.io
3. → Downloadt evaluatie-build (gratis, 30 dagen, volledig functioneel)
4. → Bouwt prototype, test met eigen PDFs
5. → Evaluatie verloopt → koopt Startup of Business tier
6. → Self-serve checkout via LemonSqueezy
7. → Ontvangt Ed25519 license file per email
8. → Plaatst naast SDK → productie-klaar
```

**Kanalen:**

| Kanaal | Actie | Prioriteit |
|--------|-------|-----------|
| xfa-sdk.com | Documentatie, pricing, download, "Get Started" | Hoog |
| crates.io | `xfa-sdk` crate (evaluation binary) | Hoog |
| npm | `@xfa/wasm` pakket | Hoog |
| PyPI | `xfa-sdk` Python package | Medium |
| GitHub | Issue tracker + voorbeeldcode (niet de SDK zelf) | Medium |
| Dev.to / Medium | Technische artikelen | Medium |
| Stack Overflow | PDF/XFA vragen beantwoorden | Medium |
| Hacker News | "Show HN: XFA 3.3 engine in pure Rust" | Hoog (eenmalig) |

### LAAG 3: Enterprise SDK + Resellers (€15K+/jaar)

**Doel**: High-value deals met grote organisaties, via resellers en directe sales.

**Hoe enterprise deals typisch starten:**
```
Developer vindt SDK
        ↓
Prototype gebouwd
        ↓
Product wordt kritiek
        ↓
Procurement betrokken
        ↓
Enterprise licentie gekocht
```

Enterprise deals beginnen bijna nooit met outbound sales. Ze beginnen bij developer adoption. Daarom is Laag 2 (self-serve SDK) de funnel voor Laag 3.

**Zie sectie 9 (Reseller & Distributie Strategie) voor details.**

---

## 8. Tijdlijn: Maand-voor-maand acties

### FASE 0: Infrastructuur (Week 1-4)

> **Doel**: Alle systemen staan klaar vóórdat er een product gelanceerd wordt.

#### Week 1-2: Juridisch & Accounts

- [ ] **KvK-inschrijving** controleren/updaten (eenmanszaak of BV)
- [ ] **EULA schrijven** voor PDFluent
  - Definitie persoonlijk vs. zakelijk gebruik
  - Disclaimer, aansprakelijkheid
  - Template via juridische dienst (Legalloyd, Rocket Lawyer NL)
- [ ] **SDK licentie-overeenkomst** schrijven
  - Per-product definitie (wat telt als "1 product")
  - Redistributieverbod voor concurrerende producten
  - Gebruiksvoorwaarden, export restricties
- [ ] **LemonSqueezy account** aanmaken
  - Store configureren
  - Producten aanmaken: PDFluent Business, SDK Startup, SDK Business
  - Checkout flows testen
  - Webhook endpoints configureren voor license file generatie
- [ ] **Moneybird account** aanmaken (voor enterprise facturatie)
  - IBAN/bankgegevens configureren
  - Factuurtemplate met bedrijfsgegevens
  - Btw-instellingen (NL btw + EU reverse charge)
- [ ] **Apple Developer Account** ($99/jaar)
- [ ] **Microsoft Store Developer Account** ($19 eenmalig)

#### Week 2-3: Licentie-infrastructuur bouwen

- [ ] **Ed25519 keypair genereren** (private key veilig opslaan!)
- [ ] **License signing tool** bouwen (CLI)
  - Input: klantdata, tier, seats, expiry, features
  - Output: signed JSON license file
  - Werkt voor zowel PDFluent als SDK licenties
- [ ] **License verifier** implementeren in Tauri (PDFluent)
  - Ed25519 signature verificatie
  - Eerste-keer-keuze dialog (Persoonlijk / Zakelijk)
  - License file laden + valideren
  - Periodieke herinnering bij zakelijk gebruik zonder license
  - Offline grace period (7 dagen na expiry)
- [ ] **License verifier** implementeren in SDK
  - Ed25519 signature verificatie bij `XfaSdk::init_with_license()`
  - 30-dagen evaluatielimiet (geen license file aanwezig)
  - Duidelijke foutmelding na evaluatieperiode
- [ ] **LemonSqueezy → license file webhook** bouwen
  - Bij betaling: automatisch license file genereren + emailen
  - Eenvoudige Cloudflare Worker of Hetzner VPS

#### Week 3-4: Website & Product

- [ ] **pdfluent.com** bouwen en live zetten
  - Landing page: screenshots, features, download buttons
  - "Gratis voor persoonlijk gebruik" prominente messaging
  - Zakelijke licentie pagina met LemonSqueezy checkout link
  - Enterprise: "10+ licenties? Neem contact op"
  - Privacy policy, EULA
  - Tech stack: Astro of Hugo op Cloudflare Pages
- [ ] **xfa-sdk.com** bouwen (of sdk.pdfluent.com)
  - Productpagina met features, benchmarks
  - Pricing pagina (transparant, Startup/Business/Enterprise/OEM)
  - "Get Started" met evaluatie-download + installatie-instructies
  - API documentatie (Docusaurus of mdBook)
  - Code voorbeelden per taal (Rust, Python, Node.js, C)
- [ ] **AlternativeTo.net listing** aanmaken
- [ ] **PDFluent + SDK builds** gereedmaken
  - Installers: .dmg (macOS), .msi (Windows), .AppImage (Linux)
  - SDK evaluatie-builds: per platform, met build-timestamp
  - Auto-update systeem opzetten (Tauri updater)
- [ ] **App Store submissions** voorbereiden

---

### FASE 1: Launch (Maand 1-3)

> **Doel**: PDFluent lanceren, eerste downloads en buzz genereren, eerste Business licenties verkopen.

#### Maand 1: Soft Launch

**Week 1-2:**
- [ ] PDFluent v1.0 publiceren op pdfluent.com
- [ ] Mac App Store submission indienen
- [ ] Microsoft Store submission indienen
- [ ] Reddit posts (r/macapps, r/windows): "I built a free Adobe Acrobat alternative in Rust — 3MB, privacy-first, no cloud"
- [ ] Dev.to blog post: "Why I Built a Free PDF Editor in Rust"
- [ ] LinkedIn post over de launch

**Week 3-4:**
- [ ] Feedback verzamelen van eerste gebruikers
- [ ] Kritieke bugs fixen
- [ ] Eerste iteratie van de app op basis van feedback
- [ ] App Store reviews monitoren en beantwoorden
- [ ] Product Hunt listing voorbereiden

**KPI targets maand 1:**
- 1.000-3.000 downloads
- 50-100 feedback items
- 0-5 Business licenties
- MRR: €0-€500

#### Maand 2: Product Hunt Launch

**Week 1: Launch day**
- [ ] **Product Hunt launch** — categorie: Productivity + Design Tools
  - Tagline: "Free Adobe Acrobat alternative — 3MB, privacy-first"
  - Eerste comment als maker: technisch verhaal, waarom Rust, waarom gratis
  - Alle netwerk mobiliseren voor upvotes op dag 1
- [ ] Hacker News post: "Show HN: PDFluent — Free 3MB PDF editor, built in Rust"
- [ ] Reddit posts in aanvullende subreddits: r/privacy, r/selfhosted, r/europrivacy, r/rust

**Week 2-4:**
- [ ] Tech journalisten benaderen met persoonlijk verhaal
  - Target: The Verge, TechCrunch, Ars Technica, Tweakers.net
  - Angle: "Solo developer bouwt gratis alternatief voor Adobe's €240/jaar product"
- [ ] YouTube tech reviewers benaderen (10K-100K subs)
- [ ] SEO-content schrijven: "PDFluent vs Adobe Acrobat", "PDFluent vs PDFgear", "Beste gratis PDF editor 2026"

**KPI targets maand 2:**
- 5.000-10.000 downloads cumulatief
- Product Hunt: top 5 van de dag
- 10-30 Business licenties
- MRR: €500-€2.000

#### Maand 3: Stabilisatie & eerste B2B

- [ ] Bug backlog afwerken op basis van user feedback
- [ ] Feature requests prioriteren (top 3 implementeren)
- [ ] XFA SDK evaluatie-build publiceren op xfa-sdk.com
- [ ] npm pakket `@xfa/wasm` publiceren
- [ ] Eerste "Powered by XFA SDK" case study schrijven (PDFluent zelf)
- [ ] LinkedIn: beginnen met regelmatig posten (2-3x/week)

**KPI targets maand 3:**
- 10.000-15.000 downloads cumulatief
- 50-100 Business licenties
- Eerste SDK evaluatie-downloads
- MRR: €1.500-€3.000

---

### FASE 2: SDK Launch & Schaling (Maand 4-6)

> **Doel**: SDK als apart product lanceren, eerste betalende SDK-klanten binnenhalen.

#### Maand 4: SDK Soft Launch

- [ ] XFA SDK documentatie site live (xfa-sdk.com/docs)
- [ ] Pricing pagina live met LemonSqueezy checkout integratie
- [ ] License file generatie-webhook live en getest
- [ ] SDK aankondiging:
  - Blog post: "Introducing XFA SDK — The first pure-Rust PDF engine"
  - Hacker News: "Show HN: Full PDF SDK in pure Rust — XFA 3.3, FormCalc, WASM"
  - r/rust post
- [ ] Eerste gratis pilots aanbieden aan 5-10 bedrijven
  - Via LinkedIn DM naar engineering leads in target verticals
  - 60 dagen gratis (extended evaluation license file)

#### Maand 5: Developer Marketing

- [ ] Technische blogposts: "XFA SDK vs Apryse", "PDF/A in Rust"
- [ ] Stack Overflow: PDF/XFA vragen beantwoorden
- [ ] crates.io: evaluatie crate publiceren
- [ ] Eerste SDK support cases afhandelen
- [ ] Cloud API MVP evalueren (scope bepalen)

#### Maand 6: Eerste betalende SDK-klanten

- [ ] Gratis pilots converteren naar betaalde licenties
- [ ] Eerste case studies schrijven
- [ ] Testimonials verzamelen
- [ ] Enterprise leads identificeren (LinkedIn Sales Navigator)
- [ ] SDK vergelijkingscontent publiceren

**KPI targets maand 6:**
- 20.000-30.000 PDFluent downloads cumulatief
- 150-300 Business licenties
- 3-5 betalende SDK-klanten
- MRR: €5.000-€8.000

---

### FASE 3: Enterprise & Resellers (Maand 7-12)

> **Doel**: Enterprise sales starten, reseller partnerships opzetten, revenue naar €10K+ MRR.

#### Maand 7-8: Enterprise Outreach

- [ ] Enterprise SDK pricing en SLA definiëren
- [ ] Security whitepaper schrijven: "Memory-Safe PDF Processing with Rust"
- [ ] PDF Association lid worden
- [ ] Eerste enterprise prospects benaderen:
  - Nederlandse overheid (Belastingdienst, RDW — XFA formulieren)
  - Duitse overheid (Finanzamt — XFA formulieren)
  - Banken en verzekeraars (compliance, security)
- [ ] LinkedIn outreach opschalen: 10-20 gesprekken per maand
- [ ] Pilot programma voor enterprise: 60 dagen gratis, dedicated support

#### Maand 9-10: Reseller Partnerships

- [ ] Eerste reseller-gesprekken:
  - SoftwareOne (Benelux/DACH)
  - Comparex/Insight
  - Regionale system integrators (Ordina, Sogeti)
- [ ] Reseller-programma opzetten (zie sectie 9)
- [ ] XFA Legacy Migratie als dienst aanbieden via system integrators

#### Maand 11-12: Consolidatie

- [ ] Eerste enterprise deals sluiten
- [ ] Revenue metrics opschonen voor investors/kopers
- [ ] Annual review: welke laag presteert het best?
- [ ] Roadmap voor jaar 2 bepalen
- [ ] Beslissing: doorgroeien, funding, of exit verkennen

**KPI targets maand 12:**
- 50.000-75.000 PDFluent downloads
- 500-800 Business licenties
- 8-15 SDK-klanten (mix van self-serve en enterprise)
- 1-2 reseller partnerships actief
- 1-3 enterprise deals (€15K+)
- MRR: €10.000-€15.000

---

### FASE 4: Schalen of Exit (Maand 13-24)

> **Doel**: Doorgroeien naar €50K+ MRR, of exit voorbereiden met sterke metrics.

#### Optie A: Doorgroeien

- [ ] Eerste hire: Developer Advocate / Community Manager
- [ ] Enterprise sales versterken (meer resellers, eigen SDR)
- [ ] Product uitbreiden:
  - Office conversie (DOCX/XLSX → PDF) — hoogst gevraagde feature
  - Mobiele SDK (iOS/Android) — grote markt
  - AI document processing — trending
- [ ] Geografische expansie: DACH, UK, Nordics, VS
- [ ] Conferenties: PDF Days (PDF Association), RustConf, local meetups

#### Optie B: Strategic Exit

- [ ] Technology brief schrijven met metrics
- [ ] Targets benaderen: Apryse, Foxit, Nutrient, Cloudflare
- [ ] M&A adviseur inhuren (5-10% commissie)
- [ ] Verwachte waardering met traction: €1M-€5M

#### Optie C: Funding

- [ ] Angel/pre-seed ronde: €200K-€500K voor 10-15% equity
- [ ] Pitch deck bouwen met metrics
- [ ] Nederlandse angel investors + accelerators

**KPI targets maand 24:**
- 150.000-300.000 PDFluent downloads
- 2.000-4.000 Business licenties
- 15-25 SDK-klanten
- 3-5 reseller partnerships
- MRR: €40.000-€60.000
- ARR: €480.000-€720.000

---

## 9. Reseller & Distributie Strategie

### Twee categorieën

#### Categorie 1: Merchant of Record — LemonSqueezy (dag 1)

| Aspect | Detail |
|--------|--------|
| Commissie | 5% + $0,50 per transactie |
| Wat zij doen | Betaling, btw-afdracht, invoicing, refunds, chargebacks, compliance |
| Wat jij doet | Product leveren, support, marketing |
| Landen | 135+ |
| Betaalmethoden | Creditcard, PayPal, Apple Pay, Google Pay, iDEAL (via Stripe) |
| Geschikt voor | PDFluent Business (1-9 seats) + SDK self-serve (Startup/Business) |
| Limiet | Niet geschikt voor enterprise procurement (geen PO, geen NET-30) |

#### Categorie 2: Enterprise Resellers & Distributeurs (maand 9+)

| Type | Voorbeelden | Commissie | Wanneer |
|------|-------------|-----------|---------|
| IT-distributeur | SoftwareOne, Comparex, Insight | 20-30% | Na 3-5 eigen klanten |
| System integrator | Sogeti, Ordina, Atos, Capgemini | 25-40% | Bij overheids-/enterprise deals |
| VAR (vertical) | Branche-specifieke partners | 20-35% | Bij vertical focus |

**Hoe enterprise resellers benaderen:**

1. **Niet beginnen zonder referenties.** Resellers willen bewezen producten.
2. Na 3-5 eigen klanten en 1-2 case studies:
   - Contact opnemen met partner managers bij SoftwareOne/Comparex
   - Pitch: "Rust PDF SDK met [X] klanten, XFA support, €5K-€200K/jaar licenties"
3. Reseller-pakket leveren:
   - Productsheet (2 pagina's)
   - Technische whitepaper
   - Demo-omgeving
   - Pricing sheet met reseller margins
   - Deal registration formulier

**XFA Legacy Migratie als reseller-product:**
System integrators (Sogeti, Ordina) die overheids-IT doen, kunnen XFA migratie-projecten verkopen:
- Jij levert de SDK + consultancy
- Zij doen de implementatie bij de klant
- Revenue split: 60% jij (SDK + expertise), 40% zij (implementatie + account)

---

## 10. Financiële Projecties

### Kosten (per jaar)

| Post | Bedrag | Frequentie |
|------|--------|-----------|
| Apple Developer Account | €99 | Jaarlijks |
| Microsoft Store | €19 | Eenmalig |
| Domein (pdfluent.com + xfa-sdk.com) | €30 | Jaarlijks |
| Hosting (Cloudflare Pages) | €0 | Gratis tier |
| Hetzner VPS (API + test infra) | €600 | Jaarlijks |
| Hetzner Storage Box | €200 | Jaarlijks |
| LemonSqueezy | 5% + $0,50/tx | Per transactie |
| Moneybird | €132 | Jaarlijks (€11/mo) |
| Juridisch (EULA, licenties) | €500-€2.000 | Eenmalig |
| LinkedIn Sales Navigator | €0-€900 | Optioneel, jaarlijks |
| **Totaal jaar 1** | **~€2.000-€4.000** | + transactiekosten |

### Revenue projectie (conservatief)

#### PDFluent Business licenties (€99/user/jaar)

| Maand | Downloads cum. | Zakelijk gebruik (5%) | Conversie (10%) | Actieve licenties | MRR |
|-------|---------------|----------------------|-----------------|-------------------|-----|
| 3 | 15.000 | 750 | 75 | 75 | €619 |
| 6 | 30.000 | 1.500 | 150 | 200 | €1.650 |
| 12 | 75.000 | 3.750 | 375 | 600 | €4.950 |
| 18 | 150.000 | 7.500 | 750 | 1.200 | €9.900 |
| 24 | 250.000 | 12.500 | 1.250 | 2.000 | €16.500 |

#### SDK licenties (per-product model)

| Maand | Evaluatie-downloads | Betalende klanten | Mix | MRR SDK |
|-------|--------------------|--------------------|-----|---------|
| 6 | 50 | 3 | 2× Startup, 1× Business | €1.150 |
| 12 | 200 | 8 | 4× Startup, 3× Business, 1× Enterprise | €3.850 |
| 18 | 500 | 15 | 6× Startup, 6× Business, 3× Enterprise | €7.900 |
| 24 | 1.000 | 22 | 8× Startup, 8× Business, 4× Enterprise, 2× OEM | €13.800 |

#### Enterprise deals (€15K+/jaar, via resellers)

| Maand | Actieve enterprise deals | Gem. deal | MRR Enterprise |
|-------|-------------------------|-----------|----------------|
| 12 | 1 | €20.000 | €1.667 |
| 18 | 3 | €25.000 | €6.250 |
| 24 | 5 | €30.000 | €12.500 |

#### Totaal gecombineerd

| Maand | MRR PDFluent | MRR SDK | MRR Enterprise | MRR Totaal | ARR |
|-------|-------------|---------|----------------|------------|-----|
| 3 | €619 | €0 | €0 | €619 | €7.425 |
| 6 | €1.650 | €1.150 | €0 | €2.800 | €33.600 |
| 12 | €4.950 | €3.850 | €1.667 | €10.467 | €125.600 |
| 18 | €9.900 | €7.900 | €6.250 | €24.050 | €288.600 |
| 24 | €16.500 | €13.800 | €12.500 | €42.800 | €513.600 |

### Break-even analyse

| Kostenpost | Maandelijks | Break-even bij |
|------------|-------------|----------------|
| Vaste kosten | ~€300/maand | 4 PDFluent Business licenties |
| LemonSqueezy fees (5%) | Variabel | Inbegrepen in marge |
| Moneybird | €11/maand | Inbegrepen in vaste kosten |
| **Totaal break-even** | **~€350/maand** | **~4 betalende klanten** |

---

## 11. Exit-opties & Waardering

### Waardering op verschillende momenten

| Moment | Metrics | Geschatte waardering | Methode |
|--------|---------|---------------------|---------|
| Nu (geen traction) | 0 klanten, 0 revenue | €200K-€500K | Cost-to-recreate |
| Maand 12 (€125K ARR) | 600+ licenties, 8 SDK klanten | €500K-€1,5M | 5-10x ARR |
| Maand 24 (€514K ARR) | 2.000+ licenties, 22 SDK klanten | €2M-€5M | 5-10x ARR |
| Maand 36 (€1M+ ARR) | 5.000+ licenties, 50+ SDK klanten | €5M-€15M | 5-15x ARR |

### Potentiële kopers

| Koper | Waarom | Geschatte bod | Kans |
|-------|--------|---------------|------|
| **Apryse** (Thoma Bravo) | Rust-stack, XFA, acquisitie-strategie | €1M-€5M | Hoog |
| **Foxit** | Concurrentie met Apryse, next-gen tech | €500K-€3M | Medium |
| **Nutrient** (PSPDFKit) | Cross-platform, Rust voor mobile | €500K-€2M | Medium |
| **Cloudflare** | WASM edge PDF processing | €1M-€5M | Medium |

### Strategische timing

**Verkoop niet te vroeg.** De waardering stijgt exponentieel met traction:
- Zonder revenue: €200K-€500K (technology asset)
- Met €100K ARR: €1M-€2M (bewezen product-market fit)
- Met €500K ARR: €3M-€7M (hogere multiple door groeitraject)

**Ideaal verkoopmoment**: 18-24 maanden, wanneer je €250K-€500K ARR hebt.

---

## 12. Risico's & Mitigatie

### Top 5 Risico's

| # | Risico | Impact | Kans | Mitigatie |
|---|--------|--------|------|-----------|
| 1 | **Overbelasting** — twee producten solo onderhouden | Hoog | Hoog | Prioriteer PDFluent stabiliteit boven nieuwe features. SDK vergt weinig support. |
| 2 | **Geen traction** — PDFluent wordt niet opgepikt | Hoog | Medium | Meerdere launch-kanalen tegelijk. Als PH faalt, focus op Reddit/HN. App Store biedt organisch bereik. |
| 3 | **Enterprise sales te traag** — lange cycles | Medium | Hoog | Niet afhankelijk van enterprise. Laag 1 en 2 draaien onafhankelijk. Enterprise is bonus. |
| 4 | **Concurrent lanceert Rust PDF SDK** | Medium | Laag | XFA + FormCalc is onze moat. Dat kopieer je niet in 6 maanden. First-mover advantage. |
| 5 | **Pricing te laag** — race to bottom | Medium | Medium | Verlaag nooit de prijs. Voeg waarde toe in plaats van korting. |

### Aanvullende risico's

| Risico | Mitigatie |
|--------|-----------|
| LemonSqueezy service issues | Backup: Paddle of FastSpring. Migratie is mogelijk. |
| Ed25519 private key compromised | Key rotation procedure: nieuwe public key in volgende app/SDK release. Oude licenties blijven werken tot expiry. Bewaar private key offline (hardware key of air-gapped machine). |
| App Store afwijzing | Direct download via website als backup. App Store is nice-to-have, niet must-have. |
| SDK piraterij | Bewust geaccepteerd risico. Focus op waarde (support, updates, compliance) niet op enforcement. Grote organisaties betalen uit compliance. |
| Juridische claim (patent troll) | Pure Rust implementatie op basis van ISO 32000 spec. Geen code gekopieerd. Bewaar documentatie van clean-room proces. |

---

## 13. Concurrentiepositie & USPs

### Onze positie in de markt

```
                    PRIJS →
                    Laag                              Hoog
                ┌───────────────────────────────────────────┐
    BREED       │  PDF.js (gratis)   │  Apryse ($15K-$210K) │
    ↑           │  PDFgear (gratis)  │  Foxit (custom)      │
    FEATURES    │                    │  Adobe/DL ($6K+)     │
    ↓           ├───────────────────────────────────────────┤
    NICHE       │  ★ PDFluent (gratis│  Nutrient ($5K/user) │
                │    persoonlijk)    │  iText ($10K-$210K)  │
                │  ★ XFA SDK (€1.99K│                      │
                │    per product)    │                      │
                └───────────────────────────────────────────┘
```

### Key Differentiators

| USP | Waarom het ertoe doet | Wie het aanspreekt |
|-----|----------------------|-------------------|
| **Enige pure-Rust XFA engine** | Geen C++ dependency, memory-safe | Security-bewuste organisaties |
| **WASM-native** | XFA in de browser, geen server nodig | SaaS bedrijven, edge computing |
| **€99/jaar vs €240/jaar** | 60% goedkoper dan Adobe Acrobat | Elke business gebruiker |
| **Per-product licensing** | Simpeler dan per-developer seats | Procurement teams |
| **Volledige XFA 3.3 + FormCalc** | Slechts 3 spelers hebben dit | Overheden, belastingdiensten |
| **3MB vs 7GB** | Instant install, geen bloat | Iedereen die Adobe haat |
| **Privacy-first, geen cloud** | Geen data naar derden | GDPR-bewuste organisaties |
| **EU compliance stack** | PDF/A, PDF/UA, ZUGFeRD | EU overheid en bedrijven |

### Concurrentie op XFA (onze sterkste moat)

| SDK | XFA Score | Status |
|-----|-----------|--------|
| **XFA-Native-Rust** | **10/10** | Volledige engine: rendering + FormCalc + reflow |
| Adobe/Datalogics | 9/10 | Forms Extension, zelfde code als Acrobat |
| Foxit | 7/10 | Native add-on: rendering + form filling |
| Syncfusion | 6/10 | .NET only: creatie + vullen + flattening |
| iText | 4/10 | Alleen flattening add-on |
| Apryse | 3/10 | Virtual Printer (Windows only) |
| Alle anderen | 0/10 | Geen XFA support |

---

## 14. KPI's & Meetbaarheid

### Maandelijkse KPI Dashboard

**PDFluent:**
| KPI | Bron | Target Maand 6 | Target Maand 12 |
|-----|------|----------------|-----------------|
| Downloads (cumulatief) | Website analytics + App Stores | 30.000 | 75.000 |
| MAU (Monthly Active Users) | Optionele telemetrie (opt-in) | 5.000 | 15.000 |
| Business licenties actief | LemonSqueezy + Moneybird | 200 | 600 |
| Conversieratio (download → business) | Berekend | 0,5% | 0,8% |
| Churn rate (business) | LemonSqueezy | <5%/maand | <3%/maand |
| App Store rating | App Stores | 4.0+ | 4.3+ |

**SDK:**
| KPI | Bron | Target Maand 6 | Target Maand 12 |
|-----|------|----------------|-----------------|
| Evaluatie-downloads | Website analytics | 50 | 200 |
| Betalende klanten | LemonSqueezy + Moneybird | 3 | 8 |
| ARR (SDK) | Berekend | €13.800 | €46.200 |
| Evaluatie → betaald conversie | Berekend | 6% | 4% (meer volume) |
| Gem. deal size | Berekend | €3.800 | €4.800 |

**Overall:**
| KPI | Target Maand 6 | Target Maand 12 | Target Maand 24 |
|-----|----------------|-----------------|-----------------|
| MRR totaal | €2.800 | €10.467 | €42.800 |
| ARR totaal | €33.600 | €125.600 | €513.600 |
| Burn rate | €300-€500/mo | €500-€1.000/mo | €1.000-€2.000/mo |
| Runway (bij €0 externe funding) | ∞ (kosten = laag) | ∞ | ∞ |

---

## 15. Beslissingslog

| # | Beslissing | Rationale | Datum |
|---|-----------|-----------|-------|
| 1 | Editor closed source (niet open source) | Geen community-management overhead, betere IP-bescherming, JetBrains-model bewezen effectief | 10-03-2026 |
| 2 | Geen feature-verschil tussen gratis en betaald | Maximale goodwill, hogere adoptie, compliance-conversie is sterker dan feature-gating | 10-03-2026 |
| 3 | LemonSqueezy als MoR voor self-serve | Ingebouwde licensing, laagste complexiteit, Stripe-backing, 5% + $0,50 | 10-03-2026 |
| 4 | Directe facturatie + bankoverschrijving voor enterprise | LemonSqueezy kan geen PO/NET-30/bankoverschrijving; enterprise verwacht dit | 11-03-2026 |
| 5 | Ed25519 signed license files (geen Keygen) | Simpeler, goedkoper, consistenter (zelfde systeem voor editor + SDK), volledig offline | 11-03-2026 |
| 6 | SDK per-product pricing (niet per-developer) | Industry-standaard, simpeler voor kopers, betere deal sizes, geen "developers tellen" | 11-03-2026 |
| 7 | SDK evaluatie: 30 dagen, geen watermark | Pragmatisch: SDK leest ook PDFs (watermark onlogisch), grote bedrijven betalen uit compliance | 11-03-2026 |
| 8 | Drie-lagen revenue model | Risicospreiding: als één laag tegenvalt, compenseren de andere twee | 10-03-2026 |
| 9 | Transparante SDK pricing op website | Vertrouwen opbouwen, self-serve mogelijk maken, Enterprise/OEM = "contact us" | 10-03-2026 |
| 10 | Resellers pas na eigen klanten (maand 9+) | Resellers willen bewezen producten met referenties | 10-03-2026 |
| 11 | Exit-optie open houden | Niet nu committeren aan doorgroeien of verkopen — metrics bepalen de keuze | 10-03-2026 |
| 12 | Moneybird voor enterprise facturatie | Nederlands, betaalbaar, btw-aangifte, professionele facturen met IBAN | 11-03-2026 |
| 13 | SDK redistributieverbod voor concurrerende producten | Industry-standaard (Apryse, iText, Nutrient), beschermt marktpositie | 11-03-2026 |

---

## Bijlage A: Checklist per fase

### Fase 0 Checklist (Week 1-4)

- [ ] KvK-registratie up-to-date
- [ ] EULA geschreven en gereviewd
- [ ] SDK licentie-overeenkomst geschreven (incl. redistributieregels)
- [ ] LemonSqueezy account + producten geconfigureerd
- [ ] Moneybird account geconfigureerd (enterprise facturatie)
- [ ] Ed25519 keypair gegenereerd (private key veilig opgeslagen)
- [ ] License signing CLI tool gebouwd
- [ ] LemonSqueezy → license file webhook werkend
- [ ] Apple Developer Account actief
- [ ] Microsoft Store Developer Account actief
- [ ] pdfluent.com live
- [ ] xfa-sdk.com live met docs + pricing
- [ ] AlternativeTo listing aangemaakt
- [ ] PDFluent license verifier geïmplementeerd in Tauri
- [ ] SDK license verifier + evaluatielimiet geïmplementeerd
- [ ] Auto-update systeem werkend
- [ ] Installers gebuild (.dmg, .msi, .AppImage)
- [ ] SDK evaluatie-builds beschikbaar per platform
- [ ] App Store submissions voorbereid

### Fase 1 Checklist (Maand 1-3)

- [ ] PDFluent v1.0 gepubliceerd op website
- [ ] Mac App Store live
- [ ] Microsoft Store live
- [ ] Product Hunt gelanceerd
- [ ] Hacker News "Show HN" gepost
- [ ] Reddit posts in 5+ subreddits
- [ ] Dev.to blog post gepubliceerd
- [ ] AlternativeTo reviews verzameld
- [ ] Eerste 10+ Business licenties verkocht
- [ ] Eerste bugs gefixt op basis van feedback
- [ ] LinkedIn profiel geoptimaliseerd
- [ ] 10+ LinkedIn posts gepubliceerd
- [ ] SDK evaluatie-build live op website

### Fase 2 Checklist (Maand 4-6)

- [ ] SDK documentatie site live
- [ ] SDK pricing pagina live met LemonSqueezy checkout
- [ ] SDK Hacker News launch
- [ ] npm pakket gepubliceerd
- [ ] 5-10 gratis SDK pilots gestart
- [ ] Eerste 3+ betalende SDK-klanten
- [ ] Eerste case study geschreven

### Fase 3 Checklist (Maand 7-12)

- [ ] Enterprise SDK pricing en SLA gedefinieerd
- [ ] Security whitepaper geschreven
- [ ] PDF Association lidmaatschap
- [ ] 10+ enterprise prospects benaderd
- [ ] Eerste reseller-gesprekken gevoerd
- [ ] Reseller-programma gedocumenteerd
- [ ] Eerste enterprise deal gesloten (via directe facturatie + bankoverschrijving)
- [ ] Maand 12 revenue review uitgevoerd
- [ ] Jaar 2 roadmap bepaald

---

## Bijlage B: Templates & Assets Nodig

| Asset | Doel | Wanneer nodig |
|-------|------|---------------|
| EULA (NL + EN) | Juridische basis | Fase 0 |
| SDK License Agreement (EN) | SDK klanten (incl. redistributieregels) | Fase 0 |
| License signing tool (CLI) | License files genereren | Fase 0 |
| Factuurtemplate (Moneybird) | Enterprise facturatie | Fase 0 |
| Product Hunt listing | Launch | Fase 1, maand 2 |
| Hacker News post | Launch | Fase 1, maand 2 |
| Productsheet SDK (2 pagina's, PDF) | Enterprise sales + resellers | Fase 2 |
| Security Whitepaper | Enterprise prospects | Fase 3 |
| Case Study template | Referenties | Fase 2+ |
| Reseller Partner Kit | Reseller onboarding | Fase 3 |
| Technology Brief (2 pagina's) | Acquisitie gesprekken | Fase 4 |
| Pitch Deck (10-15 slides) | Funding of acquisitie | Fase 4 |

---

## Bijlage C: Bronnen & Referenties

**Markt & Concurrentie:**
- [PDF SDKs Software Market Forecast 2025-2032](https://www.statsndata.org/report/pdf-sdks-software-market-298367)
- [Apryse Pricing & Licensing](https://apryse.com/pricing)
- [Nutrient SDK Pricing](https://www.nutrient.io/sdk/pricing/)
- [iText AGPL Licensing](https://itextpdf.com/how-buy/AGPLv3-license)
- [IronPDF Licensing](https://ironpdf.com/licensing/)
- [Syncfusion Pricing](https://www.syncfusion.com/sales/pricing)
- [Apryse Acquisition History](https://canvasbusinessmodel.com/blogs/brief-history/apryse-brief-history)

**Betalingsinfrastructuur:**
- [LemonSqueezy — Merchant of Record](https://www.lemonsqueezy.com/reporting/merchant-of-record)
- [LemonSqueezy Pricing](https://www.lemonsqueezy.com/pricing)
- [LemonSqueezy License Key Management](https://docs.lemonsqueezy.com/help/licensing/generating-license-keys)
- [Paddle Invoicing & B2B](https://www.paddle.com/billing/invoicing)
- [PostHog — Enterprise Software Buying](https://posthog.com/founders/how-to-buy-software-enterprise)

**Licensing & Modellen:**
- [Syncfusion Community License](https://www.syncfusion.com/products/communitylicense)
- [iText Subscription Transition](https://itextpdf.com/blog/itext-news/itext-transitions-subscription-based-commercial-licenses)

---

> **Gerelateerd document:** [LICENSING_AND_ENTERPRISE_DEPLOYMENT.md](../PDFluent/LICENSING_AND_ENTERPRISE_DEPLOYMENT.md) — technische details van het Ed25519 licentiesysteem, bestandsformaat, en enterprise deployment instructies.
>
> **Volgende update**: Na Fase 0 voltooiing — valideer aannames en pas projecties aan op basis van eerste data.
