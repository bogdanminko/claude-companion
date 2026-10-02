# Octo — пиксельный компаньон для Claude на macOS

Маленький осьминог с терминалом на пузе, который живёт поверх всех окон и по клику открывает quick entry десктопного приложения Claude.

> Неофициальный фан-проект, не связан с Anthropic.

## Что умеет

- висит поверх всех окон и на всех рабочих столах, перетаскивается мышью, запоминает позицию;
- моргает, шевелит щупальцами, краснеет при наведении, засыпает через 5 минут без внимания;
- **двойной Option** (из любого приложения) или **клик** по осьминогу — показывает капсулу с двумя кнопками:
  - ✏️ **чат** — quick entry Claude (нажимает Option + Space);
  - 〰️ **голос** — голосовой ввод Claude (имитирует Caps Lock; на экране осьминога — эквалайзер, клик по нему останавливает запись);
- если Claude не запущен — сначала запускает его;
- **правый клик** или иконка в меню-баре — меню: quick entry, открыть Claude, спрятать, уложить спать, выйти;
- работает как LaunchAgent: стартует при входе в систему и поднимается заново после падения.

## Требования

- macOS 13+
- Swift 5.10+ (Xcode или Command Line Tools)
- [Claude Desktop](https://claude.ai/download), в котором хоткей quick entry переключён на **Option + Space**
  (Settings → General → Desktop app → Quick access shortcut) — двойной Option забирает себе Octo
- для голоса — включённый в настройках Claude голосовой ввод по Caps Lock

## Установка

```bash
make install     # сборка + ~/Applications/Octo.app + LaunchAgent
```

При первом запуске macOS попросит доступ к **Accessibility** — он нужен, чтобы ловить двойной Option и нажимать хоткеи Claude.
System Settings → Privacy & Security → Accessibility → включить Octo. Перезапускать не нужно — Octo подхватит доступ сам.

> Сборка подписывается ad-hoc, поэтому после каждой переустановки доступ нужно выдать заново —
> `install.sh` сам сбрасывает устаревшую запись, так что macOS просто спросит ещё раз.

## Команды

| Команда | Что делает |
| --- | --- |
| `make run` | собрать и запустить без установки |
| `make install` | установить и зарегистрировать LaunchAgent |
| `make restart` | перезапустить агента |
| `make logs` | логи из `/tmp/octo.*.log` |
| `make uninstall` | остановить и удалить |

## Как устроено

```
Sources/Octo/
  main.swift            — точка входа, приложение без иконки в Dock
  AppDelegate.swift     — меню-бар и контекстное меню
  CompanionPanel.swift  — прозрачное плавающее окно
  CompanionView.swift   — состояние, анимации, мышь
  OctoSprite.swift      — процедурный спрайт 32×32 (форма, контур, светотень, экран)
  BubblePanel.swift     — капсула «чат / голос»
  DoubleOptionDetector.swift — глобальный двойной Option
  QuickEntry.swift      — Option + Space и Caps Lock через CGEvent
launchd/                — шаблон LaunchAgent
scripts/                — build / install / uninstall
```

`KeepAlive` настроен как `SuccessfulExit = false`: «Выйти» из меню закрывает Octo до следующего входа в систему, а падение — перезапускает.

Спрайт считается в `OctoSprite.swift` из простых фигур — меняйте формы и палитру там,
а `make preview` отрендерит все состояния в `build/preview.png` без переустановки.
