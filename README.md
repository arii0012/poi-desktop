# PoiDesktop

## Сборка на Windows (самый простой способ)

1. Установи Rust: https://rustup.rs  
   (запусти `rustup-init.exe`, нажми Enter, дождись конца, **перезапусти** терминал)

2. Если при сборке будет ошибка про `link.exe` / MSVC — установи:
   https://visualstudio.microsoft.com/visual-cpp-build-tools/  
   В установщике отметь галочку **"Desktop development with C++"**

3. Распакуй проект, дважды кликни **`build.bat`**

4. После успеха появится `poi-desktop.exe` — запускай его.

## Вручную

```
cargo build --release
target\release\poi-desktop.exe
```

## Возможности

- Фигуры + код-варианты (меняются при выборе фигуры)
- Установка обоев (работает без активации Windows)
- Возврат старых обоев
- Живой превью / статичный режим
