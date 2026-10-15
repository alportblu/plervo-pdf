# Plervo PDF — validação antes da distribuição

Atualizado em 14 de outubro de 2026. Estado: **alpha interna**. Um teste só conta como aprovado quando há resultado do commit e do pacote correspondentes; a documentação histórica do PdfCraft não comprova uma release da Plervo.

## Automação preparada

- `ci.yml`: testes, análise estática e compilação nativa no Windows, macOS e Linux; auditoria de dependências/licenças, compilação WASM e fuzzing separado.
- `candidates.yml`: execução manual ou em PRs que alteram o empacotamento, fontes adicionais fixadas por commit, testes da CLI compilada em modo release, regressões da automação local e geração de pacotes internos. Nenhuma release pública é criada automaticamente.
- No Windows, a compilação e os testes usam `x86_64-pc-windows-msvc`, CRT estático e recursos PE obrigatórios. O empacotador recebe `-SkipBuild` e utiliza esses executáveis; não recompila outra versão depois dos testes.
- Os testes existentes da CLI cobrem edição e salvamento, rotação e metadados após reabrir, renderização, extração de texto, arquivos inválidos, opções incorretas, caminhos fora da pasta permitida e preservação dos arquivos existentes. A suíte completa de CI também cobre a interface e a troca entre os 22 idiomas.
- O empacotador de macOS monta e verifica o DMG; o de Windows inspeciona as tabelas e condições do MSI. Essas verificações **não substituem instalação e uso em um sistema limpo**.

## Matriz de aceite

| Etapa | Evidência exigida | Estado da Plervo |
|---|---|---|
| Código e segurança | CI do commit aprovado; auditoria de dependências e licenças; regressões de limites, autenticação e acesso a arquivos | Verificações locais anteriores registradas; CI remoto pendente |
| PDF de ponta a ponta | Criar, abrir, editar, desfazer/refazer, salvar, reabrir, renderizar e extrair texto; original preservado; erros claros para arquivos inválidos | Fluxo sintético local no Mac aprovado na alpha anterior; repetir nos candidatos novos |
| Windows x64 | Instalar MSI em máquina/VM limpa, abrir PDF, salvar cópia, verificar atalhos, reparar e desinstalar; preservar documentos e aplicativo padrão de PDF | Pendente |
| macOS Apple Silicon | Instalar pelo DMG em ambiente limpo; abrir, editar e salvar; testar Gatekeeper e assinatura do pacote final; remover app preservando documentos | Candidato interno c8b89cdc assinado e notarizado, Gatekeeper e distribuição aprovados; instalação limpa e GUI pendentes |
| macOS Intel | Compilar e executar em Intel; repetir instalação e fluxo de PDF | Pendente; não anunciar suporte validado |
| Linux x64 | Instalar/remover DEB no Ubuntu e RPM em ambiente compatível; executar app, integração de menu, abrir e salvar PDF; validar pacote portátil | Pendente |
| Atualização | Instalar candidato anterior, alterar preferências/idioma, atualizar para versão de teste superior e verificar preservação | Pendente; requer dois pacotes distintos |
| Idiomas | Seleção automática e manual persistente; 22 catálogos; conferir caracteres, textos cortados, diálogos e direção de escrita em cada sistema | Testes de troca e apoio passaram localmente; revisão visual/linguística completa pendente |
| Distribuição | Pacotes assinados quando aplicável; notarização do Mac; varredura do pacote exato, SHA-256, notas de versão com limitações | Pendente para os novos candidatos |

Usar documentos sintéticos e contas de teste. Registrar sistema, arquitetura, commit, hash do instalador, passos e resultado. Testes de atalhos, impressão, permissões e interface precisam de execução nativa; compilação cruzada ou CLI não bastam. Uma varredura sem detecções não comprova ausência de vulnerabilidades.

## Publicação separada dos instaladores

O site estático pode ser publicado na Cloudflare Pages enquanto os downloads continuam indisponíveis. Depois do aceite de cada sistema, publicar seus instaladores no GitHub Releases e cadastrar somente URLs reais no site. O apoio usa `https://buymeacoffee.com/alport`, sem dados de pagamento no código.

O código está publicado em `alportblu/plervo-pdf` e o site em `alportblu/plervo-site`, exclusivamente na conta pessoal. O CI remoto do app está em validação; seus resultados ainda não aprovam uma distribuição. O site está hospedado no Cloudflare Pages. O Developer ID Application foi emitido para a publicadora autorizada Maintech Solutions LLC; o candidato Mac c8b89cdc passou assinatura estrita e notarização. A candidatura SignPath foi submetida pelo proprietário e o recibo confirmado; aprovação e integração Windows pendentes. Cada pacote final precisa das verificações de assinatura e dos ensaios nativos acima. Não adicionar certificados, senhas ou tokens ao repositório.

Para notarização local do Mac, `packaging/macos/package.sh` aceita `MACOS_NOTARY_PROFILE`, criado com `xcrun notarytool store-credentials`. Se `MACOS_KEYCHAIN` estiver definido, o perfil deve estar nesse Keychain, junto da identidade de assinatura. Isso permite autenticar sem colocar a senha Apple no comando de empacotamento ou no repositório. Um perfil com assinatura ad-hoc é recusado; a publicação ainda exige resultado `Accepted`, stapling e verificação do Gatekeeper. O caminho existente com as três variáveis Apple continua disponível para CI.

No commit `10a9374ff9058e9f0259bcbd7f920c7912290734`, os jobs de macOS/Linux, dependências e WASM do [CI 38651175379](https://github.com/alportblu/plervo-pdf/actions/runs/38651175379) passaram, e o job Linux de [candidatos 38651175487](https://github.com/alportblu/plervo-pdf/actions/runs/38651175487) gerou seus pacotes. Windows ainda estava em execução nessa coleta. A alteração posterior que alinha a configuração Windows precisa de sua própria execução aprovada; esses resultados não são evidência dos binários novos nem de instalação nativa.

As execuções seguintes encontraram duas falhas nos testes: casos de formulário paralelos apagavam a pasta temporária um do outro, e o teste Windows de recursos PE ainda esperava o texto anterior de autoria. Os ajustes preservam os créditos PdfCraft e Plervo e isolam cada chamada de criação da pasta. A regressão de isolamento falhou antes da correção; os 23 testes de formulário e a suíte CLI passaram localmente depois, assim como clippy, formatação, camadas, recursos e a verificação WASM de 31 crates (com avisos preexistentes de código não usado). O teste de recursos PE só executa no Windows e ainda depende do CI atualizado. Nenhuma falha foi ocultada ou teste desabilitado.

No commit `c8b89cdcc6f19881cea2be836a049ef0ac94b582`, o job Linux do CI 38658294918 falhou em três testes de busca/seleção, ainda com “Searching… 0/2” e sem camada de texto. O helper podia sair silenciosamente após 200 iterações. Agora usa o prazo de 30 segundos do helper compartilhado, exige ausência de renderizações pendentes e desenha os resultados antes de consultar a árvore acessível. Os mesmos testes passaram no Mac antes da alteração; a falha foi observada no Linux remoto. Nova execução Linux é necessária para validar a correção.

Depois do ajuste passaram 1.113 testes do crate de interface no Mac, com cinco casos já ignorados, além de formatação, clippy com avisos tratados como erros, camadas, recursos e WASM. Uma primeira execução local usou `PDFCRAFT_SYSTEM_FONTS=0`, incompatível com o teste que exige fallback de fontes; foi repetida com a configuração padrão do CI, sem modificar o produto ou publicar capturas com fontes do sistema.

O pacote Mac Apple Silicon c8b89cdc foi notarizado em 14/10/2026 (noite local): app, DMG e CLI ZIP receberam `Accepted`, sem issues nos logs Apple. App/DMG tiveram ticket anexado e validado; assinatura estrita, Gatekeeper, `syspolicy_check distribution` e a exigência de notarização da CLI passaram. Sete verificações sintéticas da CLI extraída passaram, incluindo recusa de PDF inválido e confinamento de caminhos; ClamAV verificou os artefatos finais sem detecções. Isso não comprova ausência de vulnerabilidades ou primeira execução em máquina limpa. Hashes dos candidatos internos, ainda sem publicação:

- DMG: `8ec03466ee72735aa61515a2e164d63d21a80402fa2cd5f1255c29459f786be4`.
- CLI ZIP: `6ee71ca539f6df439b372f81fb4622e981af797866d9add2b4a59ed771868cc0`.
