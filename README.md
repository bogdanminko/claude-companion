# Octo — пиксельный компаньон для Claude на macOS

Маленький осьминог с терминалом на пузе, который живёт поверх всех окон и по клику открывает quick entry десктопного приложения Claude.

> Неофициальный фан-проект, не связан с Anthropic.

## Что умеет

- висит поверх всех окон и на всех рабочих столах, перетаскивается мышью, запоминает позицию;
- моргает, шевелит щупальцами, краснеет при наведении, засыпает через 5 минут без внимания;
- **клик** — вызывает quick entry Claude (имитирует двойной Option); если Claude не запущен — запускает его;
- **правый клик** или иконка в меню-баре — меню: quick entry, открыть Claude, спрятать, уложить спать, выйти;
- работает как LaunchAgent: стартует при входе в систему и поднимается заново после падения.

## Требования

- macOS 13+
- Swift 5.10+ (Xcode или Command Line Tools)
- [Claude Desktop](https://claude.ai/download) с включённым quick entry на двойной Option

## Установка

```bash
make install     # сборка + ~/Applications/Octo.app + LaunchAgent
```

При первом запуске macOS попросит доступ к **Accessibility** — он нужен, чтобы Octo мог «нажать» двойной Option.
System Settings → Privacy & Security → Accessibility → включить Octo.

> Сборка подписывается ad-hoc, поэтому после каждой пересборки macOS может снова попросить доступ:
> выключите и включите Octo в списке Accessibility.

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
  CompanionView.swift   — спрайт, анимации, мышь
  QuickEntry.swift      — двойной Option через CGEvent
launchd/                — шаблон LaunchAgent
scripts/                — build / install / uninstall
```

`KeepAlive` настроен как `SuccessfulExit = false`: «Выйти» из меню закрывает Octo до следующего входа в систему, а падение — перезапускает.

Спрайт задан строками в `CompanionView.swift` — перерисовать персонажа можно прямо там.
