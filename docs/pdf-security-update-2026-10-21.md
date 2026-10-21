# PDF security correction — October 21, 2026

A synthetic compound vector path could retain sensitive geometry under a redaction
mark. The previous implementation clipped partially covered paths and shadings, which
hid the drawing but kept recoverable source data. Removing the clip and overlay from
the saved file recovered the original drawing. This was a confidentiality defect.

PDF alpha.7 refuses partial paths, clipping geometry under a mark, interrupted paths and
shadings, including nested forms. Fully covered ordinary painted paths can still be
removed. A refusal leaves the entire document and pending marks unchanged, including
pages processed before the refusal. A refused document has **not** been redacted and
must not be shared as though it had. Cutting vector paths and securely rewriting
shadings remain unsupported.

Regressions were observed failing before the change and passed afterward. Existing
text, image, full-save, inherited-resource, form and undo tests remain in place. Native
package qualification also exercises rejection of synthetic vector content and successful
ordinary text redaction through the installed CLI.

The installers now carry original third-party notices, including lopdf, tiny-skia,
bundled fonts and CMap data. Dependency changes invalidate the reviewed notice bundle.
See [distribution notice methodology](../packaging/licenses/README.md).

This is not a claim of zero vulnerabilities or approval for company-wide deployment.
Core document processing and bundled OCR are local, but support links, update actions
and the Suite's downloads use external services. Local processing does not eliminate
malicious-document risks. Enterprise certificate/revocation workflows and company
endpoint controls still require qualification. The existing RSA advisory exception is
based on public-key verification-only use; it is not an upstream patch.
