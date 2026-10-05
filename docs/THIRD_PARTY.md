# THIRD_PARTY — Dış kaynaklar ve lisanslar

Lisansı belirsiz hiçbir şey kullanılmaz. Yeni bağımlılık eklenince bu dosya aynı commit'te güncellenir.

## Çalışma zamanı (Rust)

| Paket                 | Lisans           | Kullanım                                    |
| --------------------- | ---------------- | ------------------------------------------- |
| tauri 2               | MIT / Apache-2.0 | Masaüstü çatısı                             |
| tauri-plugin-opener 2 | MIT / Apache-2.0 | Dış bağlantıları varsayılan tarayıcıda açma |
| serde, serde_json     | MIT / Apache-2.0 | Serileştirme                                |

## Çalışma zamanı (arayüz)

| Paket                                          | Lisans           | Kullanım |
| ---------------------------------------------- | ---------------- | -------- |
| react, react-dom 19                            | MIT              | Arayüz   |
| @tauri-apps/api 2, @tauri-apps/plugin-opener 2 | MIT / Apache-2.0 | IPC      |

## Geliştirme araçları

| Paket                                                                                                  | Lisans           |
| ------------------------------------------------------------------------------------------------------ | ---------------- |
| vite, @vitejs/plugin-react                                                                             | MIT              |
| typescript                                                                                             | Apache-2.0       |
| eslint, @eslint/js, typescript-eslint, eslint-plugin-react-hooks, eslint-plugin-react-refresh, globals | MIT              |
| prettier                                                                                               | MIT              |
| vitest, jsdom, @testing-library/*                                                                      | MIT              |
| @tauri-apps/cli                                                                                        | MIT / Apache-2.0 |
| NSIS (bundler)                                                                                         | zlib/libpng      |
| WiX Toolset 3.14 (bundler)                                                                             | MS-RL            |

## Claude Code bileşenleri

| Kaynak                                                       | Lisans                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------- |
| aitmpl / claude-code-templates (agent, skill, command, hook) | MIT                                                     |
| skill-creator (Anthropic)                                    | Apache-2.0 (`.claude/skills/skill-creator/LICENSE.txt`) |

## Fontlar, ikonlar, görseller, veri

Faz 2'den itibaren eklenecek (Sora, Manrope — OFL 1.1; ikon fontu; Ludusavi manifest — MIT; Steam CDN görselleri
yalnızca kullanıcının kendi kütüphanesi için, yeniden dağıtılmaz).
