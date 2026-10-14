# Plervo PDF

Aplicativo independente para trabalhar com PDFs no computador, baseado no [PdfCraft 0.8.0](https://github.com/storytold/pdfcraft/tree/v0.8.0).

**Estado: desenvolvimento inicial; não liberado para produção.** Windows, macOS e Linux são os alvos. Cada pacote precisa de validação no sistema correspondente antes de um download público.

Site: https://plervo.com · Apoio voluntário: https://plervo.com/apoiar/

## Desenvolvimento

Rust 1.95, dependências do sistema documentadas em `.github/workflows/ci.yml`.

```sh
cargo build --locked -p plervo-pdf -p plervo-pdf-cli -j 2
cargo run -p plervo-pdf
cargo run -p plervo-pdf-cli -- --help
cargo test --locked -p pdfcraft-ui-egui --lib control::transport_tests -j 2
cargo xtask assets
```

Os crates internos mantêm os nomes `pdfcraft-*` para facilitar rastreamento e futuras correções da base. A identidade pública, os executáveis e as configurações do Plervo são próprios. Não migramos nem alteramos automaticamente dados de PdfCraft/PrintCraft.

## Segurança e publicação

Veja [SECURITY-PLERVO.md](SECURITY-PLERVO.md). A automação continua desligada por padrão. Contribuições são opcionais e abrem uma página no navegador; não há SDK de pagamentos, cartão ou segredo financeiro no aplicativo.

O workflow de candidatos produz artefatos internos; não publica uma release automaticamente. As assinaturas de macOS/Windows e a validação nativa são etapas obrigatórias antes da distribuição pública.

## Licenças e créditos

MIT OR Apache-2.0, conforme [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) e [NOTICE](NOTICE). Créditos dos autores originais preservados. Recursos de terceiros em [ATTRIBUTION.md](ATTRIBUTION.md).

Plervo não é afiliado à Adobe ou à equipe ArtCraft. Nenhuma marca gráfica da Adobe ou ArtCraft é incluída nesta distribuição.
