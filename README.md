# md-madou-kr-patcher

메가드라이브 **마도물어 I** (Madou Monogatari I, 1996)의 한글 패치 코드베이스.

EN 패치 ROM (또는 JP ROM + IPS)에서 한글 폰트/텍스트를 삽입해 KR 패치 ROM 및 BPS 패치를 생성하는 Rust CLI 도구입니다.

## 빌드

```bash
cargo build -p madou_kr
cargo clippy -p madou_kr -- -D warnings
```

## 사용법

다음을 별도로 준비한 뒤 CLI 인자로 지정합니다:

- EN ROM (`.md`) 또는 JP ROM + IPS 패치
- TTF 폰트 (e.g. [Neo둥근모](https://github.com/neodgm/neodgm))
- `assets/` — charmap, 번역 JSON, 폰트 파일

### EN ROM에서 빌드

```bash
# 파생 파일 생성 (charmap.json, en_reference.json, text_en.json)
cargo run -p madou_kr -- init \
  --rom path/to/en_rom.md \
  --assets path/to/assets

# KR ROM 빌드 + BPS 패치 생성
cargo run -p madou_kr -- build \
  --rom path/to/en_rom.md \
  --assets path/to/assets \
  --output path/to/kr_rom.md \
  --bps path/to/patch.bps
```

### JP ROM + IPS에서 빌드

```bash
cargo run -p madou_kr -- build \
  --rom path/to/jp_rom.md \
  --ips path/to/en_patch.ips \
  --assets path/to/assets \
  --output path/to/kr_rom.md \
  --bps path/to/patch.bps
```

### QA 검증

```bash
# 제어코드 무결성 검사
cargo run -p madou_kr -- check-ctrl --assets path/to/assets

# 텍스트 오버플로우 검사
cargo run -p madou_kr -- check-overflow \
  --rom path/to/en_rom.md \
  --assets path/to/assets
```

### BPS 패치 적용

```bash
cargo run -p madou_kr -- apply \
  --rom path/to/en_rom.md \
  --patch path/to/patch.bps \
  --output path/to/kr_rom.md
```

## 라이선스

MIT License. [LICENSE](LICENSE) 참조.

이 프로젝트는 개인적·비상업적 목적으로 제작되었습니다. 게임 ROM 파일은 이 프로젝트에 포함되지 않으며, 사용자가 별도로 준비해야 합니다.
