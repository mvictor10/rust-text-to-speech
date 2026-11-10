# Changelog

## 1.0.2

- Adicionado o botão **Ouvir texto** para sintetizar e reproduzir todo o conteúdo do editor diretamente no dispositivo de áudio padrão do Windows.
- A reprodução do texto ocorre em uma thread separada para não congelar a interface gráfica.
- Adicionado o botão **Ouvir arquivo** no histórico para abrir MP3/WAV no aplicativo padrão do Windows.
- Adicionadas mensagens de status e tratamento de erros específicos para a reprodução do texto.


## 1.0.1

- Corrigido o tipo do parâmetro de canais enviado para `rusty_mp3::Mp3Encoder::push_pcm_f32`: agora utiliza `u16`, conforme a API da versão 1.0.5.
- Corrigido o encerramento da SAPI: `sapi_lite::finalize()` retorna `()`, portanto não deve ser tratado como `Result`.

## 1.0.0

- Primeira versão do projeto-fonte.
- Interface desktop em egui/eframe.
- Enumeração e seleção de vozes SAPI.
- Prévia de voz, controles de velocidade e volume.
- Exportação WAV e MP3 offline.
- Importação/exportação de texto e histórico local.
