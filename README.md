# Plervo Suite

Instale primeiro o **Plervo Suite**. A central permite instalar, abrir, atualizar e remover os apps Plervo. O instalador principal do site é o da Suite; cada editor é instalado por dentro dela.

A Suite tem interface própria em inglês, português e espanhol, com **idioma do sistema** como padrão. Seu mecanismo de instalação foi adaptado do [ArtCraft Launcher 0.4.1](docs/suite-provenance.md). O editor de PDF é baseado no [PdfCraft 0.8.0](https://github.com/storytold/pdfcraft/tree/v0.8.0), mantendo os créditos e licenças originais.

**Estado: alpha experimental; não estável.** A [alpha.4](https://github.com/alportblu/plervo-pdf/releases/tag/v0.1.0-alpha.4) oferece a Suite e os apps aprovados para Mac Apple Silicon e Linux x86_64 (DEB/RPM/TAR, glibc 2.39+). Os pacotes Mac são assinados e notarizados. A matriz de aceitação Linux verifica Ubuntu 24.04/26.04, Debian 13 e Fedora 44; Linux Mint 22 usa a base Ubuntu 24.04, mas não tem teste de desktop separado. As notas da release identificam os produtos disponíveis e as limitações verificadas. Windows tem um [instalador interno em validação](docs/windows-suite.md) e continua aguardando confirmação e integração da SignPath.

No Linux, a Suite instala cada app por usuário em `~/.local/share/plervo-suite/apps/plervo-<produto>/current` (respeitando XDG_DATA_HOME), sem senha de administrador. Downloads exigem SHA-256 e origem oficial; pacotes com links, caminhos inseguros ou tamanho excessivo são recusados. A substituição do pacote é atômica; documentos e preferências ficam fora dessa pasta e são preservados na remoção.

Site: https://plervo.com · Apoio voluntário: https://buymeacoffee.com/alport

## Idiomas

O PDF mantém os 22 idiomas da base, incluindo português do Brasil. Em **Editar → Preferências → Idioma da interface**, escolha o idioma ou use a detecção automática do sistema. A escolha é salva; trocar de idioma preserva os documentos abertos e os comandos de automação. Novas telas e ações da Plervo devem usar o mesmo sistema de tradução.

Os catálogos têm diferentes níveis de cobertura; textos ainda não traduzidos usam inglês. A revisão visual nos três sistemas, fontes para todas as escritas e o layout da direita para a esquerda continuam como critérios de lançamento. Veja [localização](docs/localization.md) e [limitações por idioma](docs/localization-parity.md).

## Desenvolvimento

Rust 1.95, dependências do sistema documentadas em `.github/workflows/ci.yml`.

```sh
cargo build --locked -p plervo-suite -p plervo-pdf -p plervo-pdf-cli -j 2
cargo run -p plervo-suite
cargo run -p plervo-suite -- --status
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
