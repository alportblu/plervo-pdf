# Upstream credits and licenses

Reviewed October 21, 2026. Plervo is maintained by AlporT and builds on the public
ArtCraft projects below. The original authors remain credited; this is a modified
distribution, independent of ArtCraft, Adobe and Autodesk.

## Project code

The source projects and Plervo modifications use **MIT OR Apache-2.0**. This means
the recipient may choose either license for the code covered by that expression.
The original `LICENSE-MIT`, `LICENSE-APACHE` and required notices must accompany
redistributed copies. A GitHub sidebar displaying only Apache-2.0 does not remove
the MIT option recorded in the manifests and license texts.

| Plervo component | Original project | Pinned source |
| --- | --- | --- |
| Suite installation engine | [ArtCraft Launcher](https://github.com/storytold/craft-launcher) | [0.4.1 / 0f295d4](https://github.com/storytold/craft-launcher/tree/0f295d4b9c90d15c2e38e36c322bdbc0fa943a54) |
| PDF | [PdfCraft](https://github.com/storytold/pdfcraft) | [0.8.0 / 9931329](https://github.com/storytold/pdfcraft/tree/9931329b9fae0b05373b4dc3ac7c58541f0891a4) |
| Plervo Photo | [photocraft](https://github.com/storytold/photocraft) | [v0.9.0 / c699e3c](https://github.com/storytold/photocraft/tree/c699e3cda69f4852b43c0b191ab0091aaeff9d6a) |
| Plervo Vector | [vectorcraft](https://github.com/storytold/vectorcraft) | [v0.12.0 / 8bc6532](https://github.com/storytold/vectorcraft/tree/8bc65329286dc1b84aa392997bb205613c3e96cb) |
| Plervo Video | [filmcraft](https://github.com/storytold/filmcraft) | [v0.8.0 / f03e215](https://github.com/storytold/filmcraft/tree/f03e21556d8e3a7155b91892d958debd5255d7af) |
| Plervo Develop | [lightcraft](https://github.com/storytold/lightcraft) | [v0.8.0 / 16dbb90](https://github.com/storytold/lightcraft/tree/16dbb90499d3c63a6582fcbba1b2550a6e2da032) |
| Plervo Motion | [effectcraft](https://github.com/storytold/effectcraft) | [v0.10.0 / 05955c0](https://github.com/storytold/effectcraft/tree/05955c08b4fd025f1a09882df72e6dbff1a8148a) |
| Plervo Publish | [designcraft](https://github.com/storytold/designcraft) | [v0.8.0 / 11a51d5](https://github.com/storytold/designcraft/tree/11a51d5b665aec8d6e675bd107a8a01514f10d00) |
| Plervo Write | [wordcraft](https://github.com/storytold/wordcraft) | [v0.7.0 / fc41918](https://github.com/storytold/wordcraft/tree/fc41918814f62645b6746ed68e71e8c2c2e85d6f) |
| Plervo Slides | [deckcraft](https://github.com/storytold/deckcraft) | [v0.7.0 / d78e657](https://github.com/storytold/deckcraft/tree/d78e657e4cb1e490ceb5445785c052d9cf5ad963) |
| Plervo Sheets | [gridcraft](https://github.com/storytold/gridcraft) | [v0.7.0 / d9d09f0](https://github.com/storytold/gridcraft/tree/d9d09f0baa912e88c5111107903fc8c640214b57) |
| Plervo CAD | [cadcraft](https://github.com/storytold/cadcraft) | [v0.7.0 / fc4e983](https://github.com/storytold/cadcraft/tree/fc4e98315a8d771b5ab832f0ade0b5e7b106df7f) |
| Plervo Audio | [soundcraft](https://github.com/storytold/soundcraft) | [v0.7.0 / 3a28a65](https://github.com/storytold/soundcraft/tree/3a28a657692c1b98b28c276d8f61df9471074f0f) |
| Plervo Canvas | [figcraft](https://github.com/storytold/figcraft) | [v0.6.0 / 833f0d2](https://github.com/storytold/figcraft/tree/833f0d2c27f0acbae5394848930ae531fc89c724) |

Exact overlay hashes and font commits are in [the source catalog](../products/sources.json).
The twelve prepared product sources were compared against their pinned upstreams:
both license files remain byte-identical, each retains attribution in `NOTICE`,
and the upstream trademark artwork has been removed. PDF and the Suite retain
their source notices in [NOTICE](../NOTICE) and [Suite provenance](suite-provenance.md).

## Dependencies and assets

The project license does not relicense third-party material. Fonts, icons, bundled
data and dependencies keep their original terms. Consult [ATTRIBUTION.md](../ATTRIBUTION.md),
its source [ATTRIBUTION.toml](../ATTRIBUTION.toml), the per-product inventories,
and the notices included in each package. Plervo artwork is original and attributed.

Packaged releases must include the original code licenses, required third-party
notices and the licenses of embedded fonts. Dependency and asset checks run before
release; Store approval does not replace these checks. Source-license verification
does not establish patent clearance or guarantee that every feature is free of bugs.
