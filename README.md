# Plervo PDF

Aplicativo independente para trabalhar com PDFs no computador, baseado no [PdfCraft 0.8.0](https://github.com/storytold/pdfcraft/tree/v0.8.0).

**Estado: desenvolvimento inicial; não liberado para produção.** Windows, macOS e Linux são os alvos. Cada pacote precisa de validação no sistema correspondente antes de um download público.

Site: https://plervo.com · Apoio voluntário: https://buymeacoffee.com/alport

## Idiomas

A interface mantém os 22 idiomas da base, incluindo português do Brasil. Em **Editar → Preferências → Idioma da interface**, escolha o idioma ou use a detecção automática do sistema. A escolha é salva; trocar de idioma preserva os documentos abertos e os comandos de automação. Novas telas e ações da Plervo devem usar o mesmo sistema de tradução.

Os catálogos têm diferentes níveis de cobertura; textos ainda não traduzidos usam inglês. A revisão visual nos três sistemas, fontes para todas as escritas e o layout da direita para a esquerda continuam como critérios de lançamento. Veja [localização](docs/localization.md) e [limitações por idioma](docs/localization-parity.md).

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
