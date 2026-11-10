# Rust Voice Studio

Aplicativo desktop Text-to-Speech para Windows 10 e Windows 11, escrito em Rust. Ele usa as vozes SAPI instaladas no próprio Windows para sintetizar fala localmente e permite salvar o resultado em MP3 ou WAV.

> **Estado deste pacote:** projeto-fonte completo, com scripts de compilação. O executável `.exe` não está incluído: este ambiente de criação não possui a toolchain Rust/MSVC para compilar e validar um binário Windows. A primeira compilação deve ser feita em um Windows com os pré-requisitos abaixo.

> **Versão 1.0.2:** adiciona o botão **Ouvir texto**, que reproduz o texto completo do editor em tempo real pelo dispositivo de áudio padrão do Windows, sem precisar exportar um arquivo.

## Recursos

- Lista as vozes SAPI instaladas no Windows.
- Exibe o nome e o identificador de idioma retornados pelo mecanismo de voz.
- Controle de velocidade SAPI de -10 a +10.
- Controle de volume SAPI de 0 a 100.
- Prévia falada pelo dispositivo de áudio padrão do Windows.
- Reprodução do texto completo do editor pelo botão **Ouvir texto**, sem criar arquivo.
- Reprodução dos arquivos MP3/WAV gerados pelo botão **Ouvir arquivo** no histórico, usando o aplicativo padrão do Windows.
- Exportação para WAV PCM, 44.1 kHz, 16 bits, mono.
- Exportação para MP3 com seleção de bitrate (96, 128, 160, 192, 256 ou 320 kbps).
- Codificação MP3 feita localmente em Rust; não usa FFmpeg nem serviços online.
- Importação e exportação de arquivos TXT/MD.
- Histórico local dos últimos 50 arquivos gerados.
- Persistência local das configurações.
- Interface gráfica desktop com egui/eframe.
- Síntese e conversão offline após a instalação das dependências e das vozes.

## Arquitetura

```text
src/
├── main.rs
├── app/
│   ├── mod.rs
│   └── ui.rs                 # Interface e coordenação das ações
├── domain/
│   ├── mod.rs
│   └── models.rs             # Modelos, configurações e formatos
└── services/
    ├── mod.rs
    ├── speech.rs             # Adaptador SAPI / geração WAV / exportação
    ├── mp3_encoder.rs        # Conversão WAV PCM para MP3
    ├── history.rs            # Persistência de histórico e configurações
    └── paths.rs              # Diretórios de dados e exportação
```

A separação é inspirada em MVC: `app` contém a camada de apresentação e coordenação, `domain` contém os modelos e `services` concentra as integrações e regras de infraestrutura.

## Pré-requisitos

1. Windows 10 ou Windows 11 de 64 bits.
2. Rust estável instalado pelo `rustup` (o arquivo `rust-toolchain.toml` solicita o alvo x64 MSVC).
3. Toolchain **MSVC** do Rust: `x86_64-pc-windows-msvc`.
4. Visual Studio Build Tools com a carga de trabalho **Desenvolvimento para desktop com C++** e Windows 10/11 SDK.
5. Pelo menos uma voz compatível com SAPI instalada no Windows.

### Instalar/confirmar o Rust

Abra o PowerShell ou o Prompt de Comando:

```powershell
rustup show
rustup default stable-x86_64-pc-windows-msvc
rustc --version
cargo --version
```

Se `rustup` ainda não estiver instalado, instale o Rust usando o instalador oficial do site `https://www.rust-lang.org/tools/install` e reinicie o terminal.

### Vozes do Windows

O aplicativo não baixa nem instala vozes. Ele enumera as vozes que o SAPI disponibiliza no computador. A disponibilidade de português do Brasil depende das vozes instaladas e registradas no Windows. Se a lista estiver vazia, verifique as configurações de idioma e fala do Windows e instale um pacote de voz compatível.

## Compilar

Extraia o ZIP para uma pasta sem restrições de escrita. Abra um terminal nessa pasta e execute:

```powershell
cargo check
cargo build --release
```

Ou execute `scripts\build_release.bat`.

O executável será criado em:

```text
target\release\rust_voice_studio.exe
```

O script de compilação copia uma versão para `dist\RustVoiceStudio.exe`.

## Executar em modo de desenvolvimento

```powershell
cargo run
```

Ou use `scripts\run_debug.bat`.

## Uso

1. Abra o Rust Voice Studio.
2. Aguarde o carregamento das vozes instaladas.
3. Digite o texto ou carregue um arquivo TXT/MD.
4. Selecione uma voz.
5. Ajuste velocidade e volume.
6. Clique em **Ouvir texto** para escutar todo o conteúdo digitado, ou em **Testar voz** para ouvir uma frase curta de demonstração.
7. Escolha MP3 ou WAV e, para MP3, defina o bitrate.
8. Clique em **Gerar áudio** e escolha o caminho/nome do arquivo.
9. No histórico, clique em **Ouvir arquivo** para abrir a gravação no reprodutor padrão do Windows.

Os arquivos são salvos por padrão em `Documentos\RustVoiceStudio\Exports`, mas o usuário pode escolher outro local na janela de exportação.

## Pastas locais de dados

As configurações e o histórico são armazenados em:

```text
%LOCALAPPDATA%\RustVoiceStudio\settings.json
%LOCALAPPDATA%\RustVoiceStudio\history.json
```

A pasta de exportação padrão é:

```text
%USERPROFILE%\Documents\RustVoiceStudio\Exports
```

## Observações técnicas

- A síntese usa a API Speech (SAPI) do Windows por meio do crate `sapi-lite`.
- Para exportar WAV, a SAPI grava um fluxo PCM de 16 bits, 44.1 kHz, mono.
- Para exportar MP3, o aplicativo gera um WAV temporário e o codifica em MP3 com `rusty_mp3`; o WAV temporário é removido após a conversão.
- A qualidade e o idioma da fala dependem da voz SAPI selecionada. A aplicação não transforma uma voz clássica em voz neural.
- A voz sintetizada por SAPI pode pronunciar textos longos de forma diferente conforme idioma, pontuação e recursos disponíveis na voz.
- Os botões **Ouvir texto** e **Testar voz** usam o dispositivo de áudio padrão do Windows e não criam arquivo. A síntese ocorre em uma thread de trabalho para manter a interface responsiva.
- A leitura de texto reproduz uma cópia do conteúdo existente no momento em que o botão foi pressionado; edições feitas depois não alteram a leitura em andamento.
- Não existe serviço em nuvem, telemetria ou envio de texto para terceiros no código deste projeto.

## Dependências principais

| Crate | Uso |
|---|---|
| `eframe` / `egui` | Interface gráfica |
| `sapi-lite` | Integração com SAPI |
| `hound` | Leitura do WAV PCM intermediário |
| `rusty_mp3` | Codificação MP3 local |
| `rfd` | Diálogos nativos de arquivos |
| `serde` / `serde_json` | Persistência de dados |
| `dirs` | Diretórios do usuário |
| `chrono` | Data/hora do histórico |

## Licenças

O código deste projeto está sob MIT (consulte `LICENSE`). As bibliotecas listadas mantêm suas próprias licenças e avisos; consulte as páginas dos respectivos crates antes de redistribuir binários.
