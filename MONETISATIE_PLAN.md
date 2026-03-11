# Monetisatieplan XFA SDK & PDFluent

> **Versie:** 1.0 — 10 maart 2026
> **Status:** Definitief strategisch plan
> **Model:** Closed-source, gratis persoonlijk gebruik, betaald zakelijk (JetBrains-model)

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

PDFluent wordt een gratis, closed-source desktop PDF editor die als credibility-motor dient voor de commerciële XFA SDK. Iedereen krijgt dezelfde volledige versie — zakelijk gebruik vereist een betaalde licentie (JetBrains-model). Omzet komt uit drie gestapelde lagen: PDFluent Business licenties, SDK self-serve verkoop, en enterprise deals via resellers.

### Kernbeslissingen

| Beslissing | Keuze | Rationale |
|------------|-------|-----------|
| Editor licentie | Closed source, gratis persoonlijk | Maximale adoptie zonder open-source overhead |
| SDK licentie | Closed source, commercieel | Volledige IP-bescherming |
| Feature-gating editor | Geen — iedereen krijgt alles | Simpelheid, goodwill, brede adoptie |
| Betalingen | LemonSqueezy (MoR) | Btw, compliance, licensing in één platform |
| Licentie-management SDK | Keygen.sh | Geavanceerd: per-developer, machine limits, Tauri plugin |
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
- Anti-SaaS sentiment groeit (TechCrunch, Bain, Chargebee)
- Geen volwaardige gratis desktop PDF editor zonder beperkingen

---

## 3. Strategisch Model: "Layered Moat"

### Drie omzetlagen die onafhankelijk werken

```
┌─────────────────────────────────────────────────────────┐
│  LAAG 3: Enterprise SDK + Resellers (€15K-€200K/deal)   │  ← maand 9+
├─────────────────────────────────────────────────────────┤
│  LAAG 2: SDK Self-Serve + Cloud API (€999-€4.999/jr)    │  ← maand 4+
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
- **Zakelijk = licentie-activatie** — via license key (LemonSqueezy)
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
│  Zakelijk: voer licentie-key in  │
└──────────────────────────────────┘
```

- Keuze wordt opgeslagen in lokale config
- "Zakelijk" toont invoerveld voor license key
- Validatie via LemonSqueezy License API (of offline cache)
- Geen harde blokkade — app werkt altijd, maar toont periodiek herinnering bij zakelijk gebruik zonder key
- Gebruiker kan altijd switchen via Instellingen

### XFA SDK (Developer Library)

**Model: Gelaagde licenties, per-developer seat**

```
XFA SDK
├── Community       → Gratis (watermark, 100 docs/maand limiet)
├── Indie           → €999/dev/jaar (bedrijven <€500K omzet)
├── Professional    → €4.999/dev/jaar (standaard)
├── Enterprise      → Custom pricing (vanaf €15.000/jaar)
└── OEM             → Custom pricing (redistributierechten)
```

**SDK Licentie-management via Keygen.sh:**
- Per-developer seat licenties
- Machine fingerprinting (max 2 machines per developer)
- Offline validatie support
- License key in code: `XfaSdk::init("LICENSE-KEY")?`
- Community tier: watermark + rate limit, geen key nodig

---

## 5. Betalingsinfrastructuur

### Architectuur

```
Klant koopt licentie
        │
        ▼
┌──────────────────┐
│  LemonSqueezy    │  ← Merchant of Record
│  (betaling +     │     Handelt af: betaling, btw, invoicing,
│   btw + invoice) │     refunds, compliance, 135+ landen
└────────┬─────────┘
         │ webhook
         ▼
┌──────────────────┐     ┌──────────────────┐
│  PDFluent:       │     │  XFA SDK:        │
│  LemonSqueezy    │     │  Keygen.sh       │
│  License API     │     │  License API     │
│  (ingebouwd)     │     │  (geavanceerd)   │
└──────────────────┘     └──────────────────┘
```

### Waarom LemonSqueezy als MoR?

| Criterium | LemonSqueezy | Paddle | FastSpring |
|-----------|-------------|--------|------------|
| Commissie | 5% + $0,50 | 5% + $0,50 | Custom (5-8%) |
| Setup-complexiteit | Zeer laag | Laag | Medium |
| Ingebouwde licensing | Ja (license keys) | Nee | Nee |
| Btw-afhandeling | Ja (135+ landen) | Ja | Ja |
| Developer-focus | Sterk | Sterk | Medium |
| Stripe-integratie | Ja (Stripe-eigendom) | Nee | Nee |
| Webhook naar Keygen | Ja (via Zapier of custom) | Ja | Ja |

**Beslissing**: LemonSqueezy voor alles.

- PDFluent Business licenties: LemonSqueezy checkout → LemonSqueezy license key → validatie in app
- SDK licenties: LemonSqueezy checkout → webhook → Keygen license aanmaken → license key naar klant

### Waarom Keygen apart voor de SDK?

LemonSqueezy's ingebouwde licensing is voldoende voor PDFluent (simpele key-validatie). Maar de SDK heeft meer nodig:
- Per-developer seat management
- Machine fingerprinting (max 2 machines)
- Offline validatie (SDK draait op build servers zonder internet)
- Usage tracking (docs/maand voor community tier)
- Rust crate (`keygen-rs`) en Tauri plugin beschikbaar

**Kosten:**
- LemonSqueezy: 5% + $0,50 per transactie (geen vaste kosten)
- Keygen: Gratis tot 100 active users, daarna ~$99/maand

### Flow: Klant koopt PDFluent Business

```
1. Klant klikt "Zakelijke licentie kopen" in app of op website
2. → LemonSqueezy checkout pagina (hosted, of in-app webview)
3. → Klant betaalt (creditcard, PayPal, iDEAL via Stripe)
4. → LemonSqueezy genereert license key automatisch
5. → Klant ontvangt key per email
6. → Klant voert key in PDFluent in
7. → App valideert via LemonSqueezy License API
8. → Licentie actief, herinnering verdwijnt
```

### Flow: Klant koopt SDK licentie

```
1. Klant kiest tier op xfa-sdk.com
2. → LemonSqueezy checkout (voor Indie/Professional)
   → Of: direct contact voor Enterprise/OEM
3. → Bij betaling: webhook naar eigen backend
4. → Backend maakt Keygen license aan
5. → Klant ontvangt Keygen license key per email
6. → Developer initialiseert SDK: XfaSdk::init("KEY")?
7. → SDK valideert via Keygen API (of offline cache)
```

---

## 6. Prijsstrategie

### PDFluent Pricing

| Gebruik | Prijs | Wat de klant krijgt |
|---------|-------|---------------------|
| Persoonlijk | **Gratis** | Volledige app, geen beperkingen |
| Zakelijk (per user) | **€99/user/jaar** | Volledige app + license key + email support |
| Zakelijk (team 10+) | **€79/user/jaar** | Volume korting + prioriteit support |
| Zakelijk (team 50+) | **Contact** | Custom pricing + onboarding |

**Pricing rationale:**
- Adobe Acrobat Pro: €240/jaar → PDFluent is 60% goedkoper
- PDF Expert: €80/jaar → PDFluent is vergelijkbaar
- PDFgear: gratis maar closed source, beperkte features
- €99/jaar is een "no-brainer" voor bedrijven — lager dan één uur IT-consultancy

### XFA SDK Pricing

| Tier | Prijs | Doelgroep | Limieten |
|------|-------|-----------|----------|
| **Community** | Gratis | Hobbyisten, evaluatie | 100 docs/maand, watermark |
| **Indie** | €999/dev/jaar | Startups (<€500K omzet) | 1 developer, 1 applicatie |
| **Professional** | €4.999/dev/jaar | Mid-market, ISVs | Per developer, ongelimiteerd |
| **Enterprise** | Vanaf €15.000/jaar | Grote organisaties | Ongelimiteerd, SLA, support |
| **OEM** | Custom | Software vendors | Redistributierechten |

**Pricing rationale:**
- IronPDF: $799-$4.799 eenmalig → wij recurring, maar lagere instap
- Nutrient: ~€5.000/user/jaar → wij vergelijkbaar op Professional
- Apryse: $15K-$210K+/jaar → wij significant goedkoper
- iText: $10K-$210K+/jaar → wij goedkoper, plus XFA support

**Publiceer pricing op de website.** Transparantie bouwt vertrouwen. Enterprise = "contact us" voor custom deals.

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
2. **Periodieke herinnering**: Na 30 dagen zakelijk gebruik zonder key → subtiele banner "Zakelijk gebruik? Activeer uw licentie."
3. **Support**: Business licentie = email support, personal = community/zelf
4. **Factuurnodig**: Bedrijven hebben een factuur nodig voor de boekhouding → LemonSqueezy levert automatisch

**Verwachte conversieratio:**
- Van alle downloads: ~5% is zakelijk gebruik
- Van zakelijk gebruik: ~10-15% koopt licentie (compliance-gedreven)
- Netto conversie: ~0,5-0,75% van totale downloads

**Target accounts:**
- MKB bedrijven die nu Adobe Acrobat betalen (€240/user/jaar → besparing van €141/user)
- Overheidsinstellingen met budgetdruk
- Non-profits en scholen (gratis of korting)
- Freelancers en ZZP'ers (€99/jaar is aftrekbaar)

### LAAG 2: SDK Self-Serve (€999-€4.999/dev/jaar)

**Doel**: Developers vinden, evalueren, en kopen de SDK zonder menselijke interactie.

**Kanalen:**

| Kanaal | Actie | Prioriteit |
|--------|-------|-----------|
| xfa-sdk.com | Documentatie, pricing, "Get Started" | Hoog |
| crates.io | `xfa-sdk` crate publiceren (community tier) | Hoog |
| npm | `@xfa/wasm` pakket publiceren | Hoog |
| PyPI | `xfa-sdk` Python package | Medium |
| GitHub | Issue tracker + voorbeeldcode (niet de SDK zelf) | Medium |
| Dev.to / Medium | Technische artikelen | Medium |
| Stack Overflow | PDF/XFA vragen beantwoorden | Medium |
| Hacker News | "Show HN: XFA 3.3 engine in pure Rust" | Hoog (eenmalig) |

**Developer experience flow:**
```
1. Developer zoekt "rust pdf sdk" of "xfa pdf processing"
2. → Vindt xfa-sdk.com of crates.io
3. → Installeert community tier (gratis, watermark)
4. → Bouwt prototype, test met eigen PDFs
5. → Wil watermark weg → koopt Indie of Professional
6. → Self-serve checkout via LemonSqueezy
7. → Ontvangt Keygen license key
8. → Productie-ready
```

### LAAG 3: Enterprise SDK + Resellers (€15K+/jaar)

**Doel**: High-value deals met grote organisaties, via resellers en directe sales.

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
  - Per-developer seat definitie
  - Gebruiksvoorwaarden, export restricties
- [ ] **LemonSqueezy account** aanmaken
  - Store configureren
  - Producten aanmaken: PDFluent Business, SDK Indie, SDK Professional
  - Checkout flows testen
  - Webhook endpoints configureren
- [ ] **Keygen.sh account** aanmaken
  - Producten aanmaken voor SDK tiers
  - Policy configureren (machine limits, offline grace period)
  - Webhook integratie met LemonSqueezy (via Zapier of custom)
- [ ] **Apple Developer Account** ($99/jaar)
- [ ] **Microsoft Store Developer Account** ($19 eenmalig)

#### Week 2-3: Website & Documentatie

- [ ] **pdfluent.com** bouwen en live zetten
  - Landing page: screenshots, features, download buttons
  - "Gratis voor persoonlijk gebruik" prominente messaging
  - Zakelijke licentie pagina met LemonSqueezy checkout link
  - Privacy policy, EULA
  - Tech stack: Astro of Hugo op Cloudflare Pages
- [ ] **xfa-sdk.com** bouwen (of sdk.pdfluent.com)
  - Productpagina met features, benchmarks
  - Pricing pagina (transparant, alle tiers)
  - "Get Started" met installatie-instructies
  - API documentatie (Docusaurus of mdBook)
  - Code voorbeelden per taal (Rust, Python, Node.js, C)
- [ ] **AlternativeTo.net listing** aanmaken (PDFluent vs Adobe, Foxit, PDF Expert)

#### Week 3-4: Product Gereedmaken

- [ ] **PDFluent migratie naar XFA SDK** afronden (of MVP bepalen)
- [ ] **License check implementeren** in Tauri
  - Eerste-keer-keuze dialog (Persoonlijk / Zakelijk)
  - License key invoerveld bij "Zakelijk"
  - LemonSqueezy License API validatie
  - Offline grace period (7 dagen)
  - Periodieke herinnering bij zakelijk gebruik zonder key (na 30 dagen)
- [ ] **Auto-update systeem** opzetten (Tauri updater)
- [ ] **Installers bouwen**: .dmg (macOS), .msi (Windows), .AppImage (Linux)
- [ ] **App Store submissions** voorbereiden (screenshots, beschrijvingen)
- [ ] **Community tier SDK** klaarmaken
  - Watermark toevoegen bij geen geldige key
  - Rate limit: 100 docs/maand
  - Publiceren op crates.io

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
- [ ] Product Hunt listing voorbereiden (screenshots, tagline, maker comment)

**KPI targets maand 1:**
- 1.000-3.000 downloads
- 50-100 GitHub issues/feedback items
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
  - Stuur ze de app + one-pager met features
- [ ] SEO-content schrijven: "PDFluent vs Adobe Acrobat", "PDFluent vs PDFgear", "Beste gratis PDF editor 2026"
- [ ] App Store Optimization: keywords, screenshots, beschrijvingen

**KPI targets maand 2:**
- 5.000-10.000 downloads cumulatief
- Product Hunt: top 5 van de dag
- 10-30 Business licenties
- MRR: €500-€2.000

#### Maand 3: Stabilisatie & eerste B2B

**Acties:**
- [ ] Bug backlog afwerken op basis van user feedback
- [ ] Feature requests prioriteren (top 3 implementeren)
- [ ] XFA SDK Community tier publiceren op crates.io
- [ ] npm pakket `@xfa/wasm` publiceren
- [ ] Eerste "Powered by XFA SDK" case study schrijven (PDFluent zelf)
- [ ] LinkedIn: beginnen met regelmatig posten over PDF, security, Rust
  - 2-3x per week, waarde-gericht (niet sales-gericht)
  - Topics: PDF security, Rust vs C++ in document processing, compliance

**KPI targets maand 3:**
- 10.000-15.000 downloads cumulatief
- 50-100 Business licenties
- Eerste SDK community tier downloads
- MRR: €1.500-€3.000

---

### FASE 2: SDK Launch & Schaling (Maand 4-6)

> **Doel**: SDK als apart product lanceren, eerste betalende SDK-klanten binnenhalen, Cloud API MVP.

#### Maand 4: SDK Soft Launch

- [ ] XFA SDK documentatie site live (xfa-sdk.com/docs)
- [ ] Pricing pagina live met LemonSqueezy checkout integratie
- [ ] Keygen integratie live (license keys voor betaalde tiers)
- [ ] SDK aankondiging:
  - Blog post: "Introducing XFA SDK — The first pure-Rust PDF engine"
  - Hacker News: "Show HN: Full PDF SDK in pure Rust — XFA 3.3, FormCalc, WASM"
  - r/rust post
  - crates.io: `xfa-sdk` update met betaalde tiers
- [ ] Eerste gratis pilots aanbieden aan 5-10 bedrijven
  - Via LinkedIn DM naar engineering leads in target verticals
  - 30 dagen gratis Professional tier

#### Maand 5: Cloud API MVP

- [ ] REST API bouwen bovenop XFA SDK
  - Endpoints: /render, /extract-text, /fill-form, /convert, /validate
  - Hosting: Cloudflare Workers (WASM) of Hetzner VPS
- [ ] API documentatie schrijven
- [ ] Usage-based pricing implementeren via LemonSqueezy
- [ ] API publiceren op RapidAPI (extra visibility)
- [ ] Eerste API-klanten werven via developer communities

#### Maand 6: Eerste betalende SDK-klanten

- [ ] Gratis pilots converteren naar betaalde licenties
- [ ] Eerste case studies schrijven
- [ ] Testimonials verzamelen
- [ ] Enterprise leads identificeren (LinkedIn Sales Navigator)
- [ ] SDK vergelijkingscontent: "XFA SDK vs Apryse", "XFA SDK vs iText"

**KPI targets maand 6:**
- 20.000-30.000 PDFluent downloads cumulatief
- 150-300 Business licenties
- 3-5 betalende SDK-klanten
- 10-20 API-gebruikers
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

- [ ] MoR (LemonSqueezy) voor self-serve tot €15K/jaar
- [ ] Eerste reseller-gesprekken:
  - SoftwareOne (Benelux/DACH)
  - Comparex/Insight
  - Regionale system integrators (Ordina, Sogeti)
- [ ] Reseller-programma opzetten:
  - Commissie: 20-30%
  - Marketing materiaal leveren
  - Demo-omgeving beschikbaar stellen
  - Deal registration systeem
- [ ] XFA Legacy Migratie als dienst aanbieden
  - Via system integrators naar overheden
  - Pricing: per-project of per-document

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
  - Downloads, ARR, klanten, churn, growth rate
  - Technische differentiatie (XFA, Rust, WASM)
- [ ] Targets benaderen:
  - Apryse/Thoma Bravo (actieve koper)
  - Foxit (zoekt next-gen tech)
  - Nutrient/PSPDFKit (cross-platform focus)
  - Cloudflare (WASM edge computing)
- [ ] M&A adviseur inhuren (5-10% commissie)
- [ ] Verwachte waardering met traction: €1M-€5M

#### Optie C: Funding

- [ ] Angel/pre-seed ronde: €200K-€500K voor 10-15% equity
- [ ] Pitch deck bouwen met metrics
- [ ] Nederlandse angel investors benaderen
- [ ] Accelerators overwegen: Y Combinator, Techstars

**KPI targets maand 24:**
- 150.000-300.000 PDFluent downloads
- 2.000-4.000 Business licenties
- 15-25 SDK-klanten
- 3-5 reseller partnerships
- MRR: €40.000-€60.000
- ARR: €480.000-€720.000

---

## 9. Reseller & Distributie Strategie

### Twee categorieën resellers

#### Categorie 1: Merchant of Record (dag 1)

**Platform: LemonSqueezy**

| Aspect | Detail |
|--------|--------|
| Commissie | 5% + $0,50 per transactie |
| Wat zij doen | Betaling, btw-afdracht, invoicing, refunds, chargebacks, compliance |
| Wat jij doet | Product leveren, support, marketing |
| Landen | 135+ |
| Betaalmethoden | Creditcard, PayPal, Apple Pay, Google Pay, iDEAL (via Stripe) |
| Geschikt voor | PDFluent Business + SDK self-serve (tot ~€15K/jaar) |

**Actie**: Account aanmaken in Fase 0.

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
| Keygen.sh | €0-€1.200 | Gratis → €99/mo bij groei |
| Juridisch (EULA, licenties) | €500-€2.000 | Eenmalig |
| LinkedIn Sales Navigator | €0-€900 | Optioneel, jaarlijks |
| **Totaal jaar 1** | **~€2.000-€5.000** | + transactiekosten |

### Revenue projectie (conservatief)

#### PDFluent Business licenties (€99/user/jaar)

| Maand | Downloads cum. | Zakelijk gebruik (5%) | Conversie (10%) | Actieve licenties | MRR |
|-------|---------------|----------------------|-----------------|-------------------|-----|
| 3 | 15.000 | 750 | 75 | 75 | €619 |
| 6 | 30.000 | 1.500 | 150 | 200 | €1.650 |
| 12 | 75.000 | 3.750 | 375 | 600 | €4.950 |
| 18 | 150.000 | 7.500 | 750 | 1.200 | €9.900 |
| 24 | 250.000 | 12.500 | 1.250 | 2.000 | €16.500 |

#### SDK licenties (gemiddeld €3.500/klant/jaar)

| Maand | Community users | Betalende klanten | Gem. deal | MRR SDK |
|-------|----------------|-------------------|-----------|---------|
| 6 | 50 | 3 | €2.500 | €625 |
| 12 | 200 | 10 | €3.000 | €2.500 |
| 18 | 500 | 18 | €3.500 | €5.250 |
| 24 | 1.000 | 25 | €4.000 | €8.333 |

#### Enterprise deals (€15K+/jaar)

| Maand | Actieve enterprise deals | Gem. deal | MRR Enterprise |
|-------|-------------------------|-----------|----------------|
| 12 | 1 | €20.000 | €1.667 |
| 18 | 3 | €25.000 | €6.250 |
| 24 | 5 | €30.000 | €12.500 |

#### Totaal gecombineerd

| Maand | MRR PDFluent | MRR SDK | MRR Enterprise | MRR Totaal | ARR |
|-------|-------------|---------|----------------|------------|-----|
| 3 | €619 | €0 | €0 | €619 | €7.425 |
| 6 | €1.650 | €625 | €0 | €2.275 | €27.300 |
| 12 | €4.950 | €2.500 | €1.667 | €9.117 | €109.400 |
| 18 | €9.900 | €5.250 | €6.250 | €21.400 | €256.800 |
| 24 | €16.500 | €8.333 | €12.500 | €37.333 | €448.000 |

### Break-even analyse

| Kostenpost | Maandelijks | Break-even bij |
|------------|-------------|----------------|
| Vaste kosten | ~€300/maand | 4 PDFluent Business licenties |
| LemonSqueezy fees (5%) | Variabel | Inbegrepen in marge |
| Keygen fees | €0-€99/maand | Vanaf ~100 SDK users |
| **Totaal break-even** | **~€400/maand** | **~5 betalende klanten** |

---

## 11. Exit-opties & Waardering

### Waardering op verschillende momenten

| Moment | Metrics | Geschatte waardering | Methode |
|--------|---------|---------------------|---------|
| Nu (geen traction) | 0 klanten, 0 revenue | €200K-€500K | Cost-to-recreate |
| Maand 12 (€110K ARR) | 600+ licenties, 10 SDK klanten | €500K-€1,5M | 5-10x ARR |
| Maand 24 (€450K ARR) | 2.000+ licenties, 25 SDK klanten | €2M-€5M | 5-10x ARR |
| Maand 36 (€1M+ ARR) | 5.000+ licenties, 50+ SDK klanten | €5M-€15M | 5-15x ARR |

### Potentiële kopers

| Koper | Waarom | Geschatte bod | Kans |
|-------|--------|---------------|------|
| **Apryse** (Thoma Bravo) | Rust-stack, XFA, acquisitie-strategie | €1M-€5M | Hoog |
| **Foxit** | Concurrentie met Apryse, next-gen tech | €500K-€3M | Medium |
| **Nutrient** (PSPDFKit) | Cross-platform, Rust voor mobile | €500K-€2M | Medium |
| **Cloudflare** | WASM edge PDF processing | €1M-€5M | Medium |
| **Datadog/New Relic** | Document compliance monitoring | €500K-€2M | Laag |

### Strategische timing

**Verkoop niet te vroeg.** De waardering stijgt exponentieel met traction:
- Zonder revenue: €200K-€500K (technology asset)
- Met €100K ARR: €1M-€2M (5-10x multiple, bewezen product-market fit)
- Met €500K ARR: €3M-€7M (hogere multiple door groeitraject)

**Ideaal verkoopmoment**: 18-24 maanden, wanneer je €250K-€500K ARR hebt én Apryse/Foxit actief acquisities doen.

---

## 12. Risico's & Mitigatie

### Top 5 Risico's

| # | Risico | Impact | Kans | Mitigatie |
|---|--------|--------|------|-----------|
| 1 | **Overbelasting** — twee producten solo onderhouden | Hoog | Hoog | Prioriteer PDFluent stabiliteit boven nieuwe features. SDK community tier vergt weinig support. |
| 2 | **Geen traction** — PDFluent wordt niet opgepikt | Hoog | Medium | Meerdere launch-kanalen tegelijk. Als PH faalt, focus op Reddit/HN. App Store biedt organisch bereik. |
| 3 | **Enterprise sales te traag** — lange cycles | Medium | Hoog | Niet afhankelijk van enterprise. Laag 1 en 2 draaien onafhankelijk. Enterprise is bonus. |
| 4 | **Concurrent lanceert Rust PDF SDK** | Medium | Laag | XFA + FormCalc is onze moat. Dat kopieer je niet in 6 maanden. First-mover advantage. |
| 5 | **Pricing te laag** — race to bottom | Medium | Medium | Verlaag nooit de prijs. Voeg waarde toe (features, support) in plaats van korting. |

### Aanvullende risico's

| Risico | Mitigatie |
|--------|-----------|
| LemonSqueezy service issues | Backup: Paddle of FastSpring. Migratie is mogelijk. |
| Keygen downtime | SDK heeft offline grace period (7 dagen). Overweeg self-hosted Keygen. |
| App Store afwijzing | Direct download via website als backup. App Store is nice-to-have, niet must-have. |
| EULA-enforcement | Niet hard afdwingen. Vertrouw op compliance-cultuur bij bedrijven. Focus op waarde, niet op dwang. |
| Juridische claim (patent troll) | Rustig blijven. Pure Rust implementatie op basis van ISO 32000 spec. Geen code gekopieerd. Bewaar documentatie van clean-room proces. |

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
                │  ★ XFA SDK (€999+)│                      │
                └───────────────────────────────────────────┘
```

### Key Differentiators

| USP | Waarom het ertoe doet | Wie het aanspreekt |
|-----|----------------------|-------------------|
| **Enige pure-Rust XFA engine** | Geen C++ dependency, memory-safe | Security-bewuste organisaties |
| **WASM-native** | XFA in de browser, geen server nodig | SaaS bedrijven, edge computing |
| **€99/jaar vs €240/jaar** | 60% goedkoper dan Adobe Acrobat | Elke business gebruiker |
| **Volledige XFA 3.3 + FormCalc** | Slechts 3 spelers hebben dit (Adobe, Foxit, wij) | Overheden, belastingdiensten |
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
| Business licenties actief | LemonSqueezy dashboard | 200 | 600 |
| Conversieratio (download → business) | Berekend | 0,5% | 0,8% |
| Churn rate (business) | LemonSqueezy | <5%/maand | <3%/maand |
| App Store rating | App Stores | 4.0+ | 4.3+ |
| Support tickets/maand | Email/issue tracker | <50 | <100 |

**SDK:**
| KPI | Bron | Target Maand 6 | Target Maand 12 |
|-----|------|----------------|-----------------|
| Community tier users | Keygen/crates.io | 50 | 200 |
| Betalende klanten | LemonSqueezy + Keygen | 3 | 10 |
| ARR (SDK) | Berekend | €7.500 | €30.000 |
| Trial → betaald conversie | Keygen | 10% | 15% |
| Gem. deal size | LemonSqueezy | €2.500 | €3.000 |

**Overall:**
| KPI | Target Maand 6 | Target Maand 12 | Target Maand 24 |
|-----|----------------|-----------------|-----------------|
| MRR totaal | €2.275 | €9.117 | €37.333 |
| ARR totaal | €27.300 | €109.400 | €448.000 |
| Burn rate | €300-€500/mo | €500-€1.000/mo | €1.000-€2.000/mo |
| Runway (bij €0 externe funding) | ∞ (kosten = laag) | ∞ | ∞ |

### Kwartaalrapportage

Elk kwartaal:
1. Revenue per laag (PDFluent / SDK / Enterprise)
2. Download trends en conversieratio's
3. Churn analyse (waarom vertrekken klanten?)
4. Top feature requests (prioriteren voor roadmap)
5. Concurrentie-update (nieuwe spelers, prijswijzigingen)
6. Exit-waarde schatting (ARR × multiple)

---

## 15. Beslissingslog

Dit document bevat de strategische beslissingen die zijn genomen. Gebruik deze log om terug te verwijzen bij twijfel.

| # | Beslissing | Rationale | Datum |
|---|-----------|-----------|-------|
| 1 | Editor closed source (niet open source) | Geen community-management overhead, betere IP-bescherming, JetBrains-model bewezen effectief | 10-03-2026 |
| 2 | Geen feature-verschil tussen gratis en betaald | Maximale goodwill, hogere adoptie, compliance-conversie is sterker dan feature-gating | 10-03-2026 |
| 3 | LemonSqueezy als MoR | Ingebouwde licensing, laagste complexiteit, Stripe-backing, 5% + $0,50 | 10-03-2026 |
| 4 | Keygen voor SDK licensing | Tauri plugin, machine fingerprinting, offline validatie, Rust crate beschikbaar | 10-03-2026 |
| 5 | Drie-lagen revenue model | Risicospreiding: als één laag tegenvalt, compenseren de andere twee | 10-03-2026 |
| 6 | Transparante SDK pricing op website | Vertrouwen opbouwen, self-serve mogelijk maken, enterprise = "contact us" | 10-03-2026 |
| 7 | Resellers pas na eigen klanten (maand 9+) | Resellers willen bewezen producten met referenties | 10-03-2026 |
| 8 | Exit-optie open houden | Niet nu committeren aan doorgroeien of verkopen — metrics bepalen de keuze | 10-03-2026 |

---

## Bijlage A: Checklist per fase

### Fase 0 Checklist (Week 1-4)

- [ ] KvK-registratie up-to-date
- [ ] EULA geschreven en gereviewd
- [ ] SDK licentie-overeenkomst geschreven
- [ ] LemonSqueezy account + producten geconfigureerd
- [ ] Keygen account + producten geconfigureerd
- [ ] LemonSqueezy → Keygen webhook integratie werkend
- [ ] Apple Developer Account actief
- [ ] Microsoft Store Developer Account actief
- [ ] pdfluent.com live
- [ ] xfa-sdk.com live met docs
- [ ] AlternativeTo listing aangemaakt
- [ ] PDFluent license check geïmplementeerd in Tauri
- [ ] Auto-update systeem werkend
- [ ] Installers gebuild (.dmg, .msi, .AppImage)
- [ ] App Store submissions voorbereid
- [ ] SDK community tier op crates.io

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
- [ ] LinkedIn profiel geoptimaliseerd voor PDF/SDK thought leadership
- [ ] 10+ LinkedIn posts gepubliceerd

### Fase 2 Checklist (Maand 4-6)

- [ ] SDK documentatie site live
- [ ] SDK pricing pagina live
- [ ] SDK Hacker News launch
- [ ] npm pakket gepubliceerd
- [ ] 5-10 gratis pilots gestart
- [ ] Cloud API MVP live
- [ ] Eerste 3+ betalende SDK-klanten
- [ ] Eerste case study geschreven
- [ ] API op RapidAPI

### Fase 3 Checklist (Maand 7-12)

- [ ] Enterprise SDK pricing en SLA gedefinieerd
- [ ] Security whitepaper geschreven
- [ ] PDF Association lidmaatschap
- [ ] 10+ enterprise prospects benaderd
- [ ] Eerste reseller-gesprekken gevoerd
- [ ] Reseller-programma gedocumenteerd
- [ ] Eerste enterprise deal gesloten
- [ ] Maand 12 revenue review uitgevoerd
- [ ] Jaar 2 roadmap bepaald

---

## Bijlage B: Templates & Assets Nodig

| Asset | Doel | Wanneer nodig |
|-------|------|---------------|
| EULA (NL + EN) | Juridische basis | Fase 0 |
| SDK License Agreement (EN) | SDK klanten | Fase 0 |
| Product Hunt listing (screenshots + copy) | Launch | Fase 1, maand 2 |
| Hacker News post (titel + eerste comment) | Launch | Fase 1, maand 2 |
| Productsheet SDK (2 pagina's, PDF) | Enterprise sales + resellers | Fase 2 |
| Security Whitepaper | Enterprise prospects | Fase 3 |
| Case Study template | Referenties | Fase 2+ |
| Reseller Partner Kit | Reseller onboarding | Fase 3 |
| Technology Brief (2 pagina's) | Acquisitie gesprekken | Fase 4 |
| Pitch Deck (10-15 slides) | Funding of acquisitie | Fase 4 |

---

## Bijlage C: Bronnen & Referenties

- [PDF SDKs Software Market Forecast 2025-2032](https://www.statsndata.org/report/pdf-sdks-software-market-298367)
- [Apryse Pricing & Licensing](https://apryse.com/pricing)
- [Nutrient SDK Pricing](https://www.nutrient.io/sdk/pricing/)
- [iText AGPL Licensing Discussion](https://beemanmuchmore.com/software-licensing-trolls-apryse-and-itext/)
- [LemonSqueezy — Merchant of Record](https://www.lemonsqueezy.com/reporting/merchant-of-record)
- [LemonSqueezy Pricing](https://www.lemonsqueezy.com/pricing)
- [LemonSqueezy 2026 Stripe Update](https://www.lemonsqueezy.com/blog/2026-update)
- [LemonSqueezy License Key Management](https://docs.lemonsqueezy.com/help/licensing/generating-license-keys)
- [Keygen.sh — Software Licensing API](https://keygen.sh)
- [Keygen for Tauri Apps](https://keygen.sh/for-tauri-apps/)
- [Keygen Pricing](https://keygen.sh/pricing/)
- [tauri-plugin-keygen (Rust crate)](https://crates.io/crates/tauri-plugin-keygen)
- [Open Core Business Model Handbook](https://handbook.opencoreventures.com/open-core-business-model/)
- [Open Source Monetization Strategies](https://www.reo.dev/blog/monetize-open-source-software)
- [Open Source Business Models (Wikipedia)](https://en.wikipedia.org/wiki/Business_models_for_open-source_software)
- [Open Source Models That Work in 2026](https://technews180.com/blog/open-source-models-that-work/)
- [Indie Hacker Success Stories 2026](https://www.somethingsblog.com/2026/01/24/real-indie-hacker-success-stories-that-prove-its-still-possible-in-2026/)
- [How to Hit $10K MRR in 2026](https://dev.to/shayy/how-to-actually-hit-10k-mrr-in-2025-no-bs-just-what-works-204k)
- [SaaSpocalypse — TechCrunch](https://techcrunch.com/2026/03/01/saas-in-saas-out-heres-whats-driving-the-saaspocalypse/)
- [Will AI Disrupt SaaS? — Bain & Company](https://www.bain.com/insights/will-agentic-ai-disrupt-saas-technology-report-2025/)
- [Apryse Acquisition History](https://canvasbusinessmodel.com/blogs/brief-history/apryse-brief-history)
- [Apryse acquires LEAD Technologies](https://www.thomabravo.com/press-releases/apryse-announces-acquisition-of-ai-powered-document-toolkit-provider-lead-technologies)

---

> **Volgende update**: Na Fase 0 voltooiing — valideer aannames en pas projecties aan op basis van eerste data.
