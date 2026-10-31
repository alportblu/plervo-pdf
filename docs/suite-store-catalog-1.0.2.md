# Suite Store catalog 1.0.2.0 — release hold

This candidate adds the ten remaining reserved product identities to the Suite catalog and its trusted installed-app inventory. It uses Microsoft Store for acquisition and updates; it does not download or execute external installers.

**Do not merge, submit or publish this candidate until all ten additional apps have completed certification and Partner Center confirms they are publicly available.** Reservations, IARC ratings, successful internal MSIX builds and submission receipts are insufficient.

Suite 1.0.1.0 is currently published and offers PDF, CAD and Video. This change is an internal candidate only until the publication hold is cleared.

Before release:

- Verify the live listing and exact MSIX identity for Photo, Vector, Develop, Motion, Publish, Write, Slides, Sheets, Audio and Canvas.
- Complete normal required CI and native Suite MSIX installation, UI, upgrade, removal and all thirteen acquisition-link checks. Preserve the refusal of unrecognized products, arbitrary URLs and external uninstall actions.
- Review actual English, Portuguese and Spanish screenshots; compare the production package with the tested payload and perform the final malware scan.
- Submit only the production 1.0.2.0 package. The validation-only 1.0.3.0 package exercises upgrade handling and must never be distributed.

The native CI runner cannot exercise consumer Microsoft Store acquisition or activation of an app signed by Store. Those limitations remain explicit in the evidence.
