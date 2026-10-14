# Segurança do Plervo PDF

Base fixada: PdfCraft v0.8.0, commit `9931329b9fae0b05373b4dc3ac7c58541f0891a4`.
Primeira versão do fork: em desenvolvimento. Não existe instalador público aprovado.

## Alterações desta etapa

- `rustls` 0.23.45 → 0.23.46, removendo os quatro alertas RustSec identificados na auditoria de 14/10/2026.
- Automação local continua opt-in (`--control`), bind em `127.0.0.1`, token aleatório por sessão e arquivo de controle restrito ao proprietário.
- No máximo 16 conexões ativas; a vaga é devolvida quando o manipulador termina, inclusive em falhas de criação de thread.
- Autenticação deve terminar em cinco segundos. Linhas vazias e bytes enviados lentamente não renovam esse prazo.
- Requisições autenticadas têm no máximo 1 MiB e 30 segundos para leitura; antes da autenticação, 4 KiB.
- Escritas têm timeout de cinco segundos. Uma chamada sem resposta da interface por 30 segundos encerra a conexão.
- Fila de automação limitada a 32 pedidos; saturação retorna erro sem bloquear a interface.
- Identidade, pasta de configurações, MSI UpgradeCode e URLs próprios; o fork não migra dados de PrintCraft/PdfCraft.
- Apoio voluntário por link HTTPS fixo `https://plervo.com/apoiar/`, aberto somente por ação do usuário. Nenhum widget de pagamento ou segredo é incorporado ao aplicativo.
- Empacotamento AppImage não baixa nem executa automaticamente uma ferramenta sem versão/hash verificados.
- Workflows próprios, com permissões de leitura e ações fixadas por SHA. Builds de candidatos não publicam releases.

## RSA: alerta residual, mitigação existente preservada

`RUSTSEC-2023-0071` afeta operações RSA com chave privada na biblioteca `rsa`; não há correção indicada no banco consultado. A base usa essa biblioteca apenas para operações públicas de verificação (`crates/sign/src/rsa_pad.rs`, `ec512.rs`). Operações nativas com chave privada usam `aws-lc-rs` ou os provedores do sistema; no navegador, o caminho privado RSA é recusado. A exceção documentada em `deny.toml` é mantida, sem ocultar o achado do relatório bruto de `cargo audit`.

Isso não demonstra ausência de outras falhas. Alterações futuras nas dependências e no código de assinaturas precisam reavaliar essa mitigação. Não importar `rsa::RsaPrivateKey` nem substituir os provedores nativos por operações privadas dessa biblioteca.

## Validação antes de distribuir

1. Testes das alterações, criptografia e fluxos de abrir/alterar/salvar/reabrir PDFs sintéticos.
2. Auditoria de dependências atualizada, análise dos avisos e varredura dos arquivos finais.
3. Testar instalação, atualização, desinstalação e preservação de dados em cada sistema e arquitetura declarados.
4. macOS: Developer ID próprio, hardened runtime, notarização e stapling. Apple Development/Apple Distribution não substituem Developer ID para esse canal.
5. Windows: assinatura Authenticode própria e teste real do MSI. Linux: pacote e checksum vinculados à release oficial; definir assinatura de publicação.
6. SHA-256 dos artefatos, notas de versão e canal privado de segurança publicados antes de ativar downloads.
7. Só então atualizar os links de download do site. Não remover quarentena, desativar Gatekeeper ou orientar usuários a ignorar proteções.

Não há atualização automática nem download/execução automática de instaladores; a consulta de versões continua sendo iniciada pelo usuário.
