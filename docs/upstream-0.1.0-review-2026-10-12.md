# tachyon-core v0.1.0: рекомендации для Rust X

Анализ выполнен по release tag `v0.1.0` (`8eb4584`), предыдущему
upstream `2f417b8` (v0.0.9) и Rust X main `99abc54`.
Последующий upstream main `1ea2f4b` уже сокращает CI; это не часть
рекомендованного переноса. GitHub сообщает publication 2026-10-12 20:33:32 UTC.
Исходники продукта, версии, пакеты и роутеры не изменялись.

Источники:
- https://github.com/Dushnilin/tachyon-core/releases/tag/v0.1.0
- https://github.com/Dushnilin/tachyon-core/compare/v0.0.9...v0.1.0
- https://github.com/XTLS/Xray-core/blob/main/proxy/proxy.go

## Переносить адресно в первую очередь

1. **HTTP/2 backpressure, часть `a572c60`.** Наш H2Stream возвращает
   `release_capacity` при получении chunk, до его потребления. Медленный
   читатель позволяет очереди продолжать расти. Upstream возвращает окно при
   чтении/пропуске данных и освобождает остаток на Drop. В gRPC отдельно
   учитывает потреблённый заголовок. Перенести эту логику, сохранив полный учёт
   окна при Pending, partial read, skip, error и Drop. Увеличение окна не входит
   в необходимое исправление: проверить реальные константы нашей версии и
   сохранить разумный предел памяти. Проверки: быстрый отправитель/медленный
   читатель, остановка отправителя до дренажа, возобновление после дренажа,
   несколько потоков одного соединения и закрытие непрочитанного потока.

2. **Лишние записи cache.db и TSRS, `48eec78` + `1dbda76`.** Полезны
   сравнение старого/нового значения в put/apply_changes, отсутствие dirty при
   no-op, отказ от повторной записи неизменного TSRS и Clash Mode, батчинг
   обновления timestamp вместо принудительного flush после HTTP 304.
   У нас парная запись FakeIP через apply_changes уже реализована: переносить
   новые no-op/repair части, не заменять её целиком. Проверить реальное
   количество flush/изменение mtime: upstream тест с названием
   memory_and_disk_hits_do_not_dirty_the_cache_or_write_to_flash проверяет
   восстановление данных, но не измеряет записи. Сравнение больших TSRS через
   fs::read создаёт дополнительный полный Vec: для роутеров предпочтителен
   bounded/streaming compare. Сохранить атомарную замену и обработку ошибок.

3. **Clash API, `6a3dc7d`.** /traffic сейчас отдаёт накопленные totals;
   для графика скорости нужны разницы счётчиков. Upstream использует дельту
   за тик 1 s: при задержке event loop правильнее делить на фактически
   прошедшее время. /connections.memory можно заполнить RSS; Linux page size
   получать у системы. Сохранить totals в /connections. Проверки: две
   последовательные выборки, отсутствие трафика, задержанный тик и единицы RSS.

## Полезно следующим отдельным этапом

- **DNS из `d75ed1a`:** ограничить UDP retry, переиспользовать DoH H2,
  параллельно разрешать A/AAAA с ограниченным ожиданием второго семейства,
  сохранять последний адрес DNS-сервера при неудачном refresh. Переносить
  отдельными изменениями, проверив timeout/cancellation, reconnect, TLS/SNI,
  bootstrap/detour и отсутствие DNS-петли. Serve-stale, fastest-IP и DoH
  interception — самостоятельные функции; автоматически не включать.
- **FakeIP startup repair (`48eec78`):** raw binary keys, восстановление
  отсутствующей reverse записи, удаление orphan/conflicting/out-of-range
  записей. Сохранить наши persistence и eviction tests; добавить mixed IPv4/
  IPv6, повреждённые записи, cache больше RAM LRU, повторный restart и курсоры.
  Парная batch-запись не означает fsync каждого DNS-запроса: durability
  остаётся ограничена flush policy.
- **WireGuard lifecycle, часть `d75ed1a`:** считать срок ключа от начала
  сессии, закрывать сокет после caller EOF, учитывать smoltcp poll_at и подавать
  application data до poll. Отделить от kernel WireGuard и роста буферов.
  Проверки: EOF/отмена probes, освобождение портов, rekey при keepalive,
  активный обмен и отсутствие busy loop.
- **TUN correctness, часть `d75ed1a`:** сохранять хвост partial send_slice и
  ограничить очереди. Не копировать буквально spawn на каждый Full в
  poll_write: повторные polls могут создавать несколько ожидающих задач.
  Нужен один отслеживаемый reservation/waker с отменой. Проверки: точная
  доставка байтов при коротких записях, full channel, Drop и ограничение памяти.
- **TLS pin parsing, часть `13cd676`:** hex/base64/prefix support полезен,
  но изменение доверия рассматривать отдельно. Upstream меняет CA+pin на
  pin-only и AND двух pin-полей на OR. Это изменение политики, а не только
  форматирование. Согласовать с нашей конфигурацией и эталонной спецификацией;
  проверить неверный pin, оба поля, self-signed и proof of key possession.

## Не переносить буквально

- **Detour (`70b49b3`).** connect_index/connect_udp_index вызывают connect
  только у результата resolve_detour_chain. Пример first SOCKS -> second SOCKS
  -> direct фактически подключается через second и не строит туннель к first.
  Это выбор последнего outbound, а не каскад прокси. Unknown detour также
  молча завершает обход, а имена direct/bypass/default трактуются особо.
  Нужен отдельный dial-through API и wire test с двумя серверами,
  доказывающий участие обоих hops. close_by_tag общего exit может прервать
  чужие соединения другого selector: закрытие должно учитывать принадлежность
  сессии изменённой группе и поколению маршрута. Сохранить нашу route revalidation.
- **Vision UUID removal (`13cd676`).** Upstream удаляет UUID первого
  padded frame и меняет reader. Эталон Xray XtlsPadding отправляет UUID один
  раз; XtlsUnpadding распознаёт его перед пятибайтовым заголовком. Наличие UUID
  также в VLESS request не делает его лишним в Vision framing. Нашу реализацию
  сохранить; upstream unit tests переписаны под новый формат и не доказывают
  совместимость с Xray.
- **Увеличение буферов.** smoltcp 64 -> 256 KiB на каждое направление
  означает ещё 384 KiB на TCP socket; 100 таких sockets — около 37.5 MiB
  дополнительно. Это относится к userspace-stack sockets, а не ко всем
  обычным TProxy потокам. H2 окна также не увеличивать ради рекламного
  throughput без RSS/latency измерений на целевых устройствах.
- **Удаление mimalloc (`0ba54b1`).** Системный allocator не гарантирует
  меньший RSS, особенно на musl. Сначала сравнить idle/active/после закрытия,
  fragmentation, CPU и throughput. Заявление Pure Rust само по себе не
  обосновывает смену allocator.
- **Самописный BLAKE3 (`a572c60`).** Не заменять поддерживаемый crate без
  необходимости. Исправление бинарного derive-key context отделить от новой
  криптографии и проверить совместимым API/эталонными vectors.
- **Upstream CI и release metadata.** Не импортировать сокращение CI,
  upstream версии, сборки и упаковку в наш подписанный workflow. Мелкие
  allocation fixes допустимы позже; excluded legacy protocols и benchmark
  scripts не являются первоочередной интеграцией.

## Порядок работ и пределы проверки

Рекомендуемые небольшие задачи: H2 backpressure -> cache no-op/TSRS -> Clash
metrics -> DNS transport -> FakeIP repair -> WireGuard/TUN lifecycle.
Сохранить Router::prepare, acknowledged reload и TCP/UDP route revalidation.
Предыдущие рекомендации по Mux partial write и compound duration из
tachyon-releases-review-2026-10-12.md остаются отдельными открытыми задачами.

Выполнены GitHub API lookup, fetch без перезаписи собственных тегов,
статический diff/история и сравнение Vision с Xray source. Cargo в текущем
PowerShell PATH отсутствует; runtime tests и benchmark не запускались.
Для внедрения нужны focused runtime tests и Rust X profile checks из нашего
CI; интеграция — на существующих VMware OpenWrt после фиксации состояния.
Это рекомендация по коду, не заявление о проверенной работе нового upstream
на наших роутерах.
