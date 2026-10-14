# Plervo PDF — validação antes da distribuição

Atualizado em 14 de outubro de 2026. Estado: **alpha interna**. Um teste só conta como aprovado quando há resultado do commit e do pacote correspondentes; a documentação histórica do PdfCraft não comprova uma release da Plervo.

## Automação preparada

- `ci.yml`: testes, análise estática e compilação nativa no Windows, macOS e Linux; auditoria de dependências/licenças, compilação WASM e fuzzing separado.
- `candidates.yml`: execução manual ou em PRs que alteram o empacotamento, fontes adicionais fixadas por commit, testes da CLI compilada em modo release, regressões da automação local e geração de pacotes internos. Nenhuma release pública é criada automaticamente.
- Os testes existentes da CLI cobrem edição e salvamento, rotação e metadados após reabrir, renderização, extração de texto, arquivos inválidos, opções incorretas, caminhos fora da pasta permitida e preservação dos arquivos existentes. A suíte completa de CI também cobre a interface e a troca entre os 22 idiomas.
- O empacotador de macOS monta e verifica o DMG; o de Windows inspeciona as tabelas e condições do MSI. Essas verificações **não substituem instalação e uso em um sistema limpo**.

## Matriz de aceite

| Etapa | Evidência exigida | Estado da Plervo |
|---|---|---|
| Código e segurança | CI do commit aprovado; auditoria de dependências e licenças; regressões de limites, autenticação e acesso a arquivos | Verificações locais anteriores registradas; CI remoto pendente |
| PDF de ponta a ponta | Criar, abrir, editar, desfazer/refazer, salvar, reabrir, renderizar e extrair texto; original preservado; erros claros para arquivos inválidos | Fluxo sintético local no Mac aprovado na alpha anterior; repetir nos candidatos novos |
| Windows x64 | Instalar MSI em máquina/VM limpa, abrir PDF, salvar cópia, verificar atalhos, reparar e desinstalar; preservar documentos e aplicativo padrão de PDF | Pendente |
| macOS Apple Silicon | Instalar pelo DMG em ambiente limpo; abrir, editar e salvar; testar Gatekeeper e assinatura do pacote final; remover app preservando documentos | DMG interno anterior verificado; instalação final e notarização pendentes |
| macOS Intel | Compilar e executar em Intel; repetir instalação e fluxo de PDF | Pendente; não anunciar suporte validado |
| Linux x64 | Instalar/remover DEB no Ubuntu e RPM em ambiente compatível; executar app, integração de menu, abrir e salvar PDF; validar pacote portátil | Pendente |
| Atualização | Instalar candidato anterior, alterar preferências/idioma, atualizar para versão de teste superior e verificar preservação | Pendente; requer dois pacotes distintos |
| Idiomas | Seleção automática e manual persistente; 22 catálogos; conferir caracteres, textos cortados, diálogos e direção de escrita em cada sistema | Testes de troca e apoio passaram localmente; revisão visual/linguística completa pendente |
| Distribuição | Pacotes assinados quando aplicável; notarização do Mac; varredura do pacote exato, SHA-256, notas de versão com limitações | Pendente para os novos candidatos |

Usar documentos sintéticos e contas de teste. Registrar sistema, arquitetura, commit, hash do instalador, passos e resultado. Testes de atalhos, impressão, permissões e interface precisam de execução nativa; compilação cruzada ou CLI não bastam. Uma varredura sem detecções não comprova ausência de vulnerabilidades.

## Publicação separada dos instaladores

O site estático pode ser publicado na Cloudflare Pages enquanto os downloads continuam indisponíveis. Depois do aceite de cada sistema, publicar seus instaladores no GitHub Releases e cadastrar somente URLs reais no site. O apoio usa `https://buymeacoffee.com/alport`, sem dados de pagamento no código.

O código está publicado em `alportblu/plervo-pdf` e o site em `alportblu/plervo-site`, exclusivamente na conta pessoal. O primeiro CI remoto do app foi iniciado; seus resultados ainda não aprovam uma distribuição. O site está hospedado no Cloudflare Pages. Para a distribuição final, faltam as identidades de assinatura apropriadas de macOS/Windows e os ensaios nativos acima. Não adicionar certificados, senhas ou tokens ao repositório.
