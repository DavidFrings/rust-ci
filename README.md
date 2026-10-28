# rust-ci

Zentrale, wiederverwendbare GitHub-Actions-Workflows für alle meine Rust-Projekte.
Die Projekte rufen sie mit wenigen Zeilen auf (siehe das Repo `rust-template`), die Prüfungen leben nur hier.

## Benutzung

`.github/workflows/ci.yml` im Projekt:

```yaml
jobs:
  rust:
    uses: DavidFrings/rust-ci/.github/workflows/rust.yml@v1
    with:
      msrv: "1.85"
```

`.github/workflows/security.yml` im Projekt (mit `deny.toml` im Repo-Root):

```yaml
jobs:
  deny:
    uses: DavidFrings/rust-ci/.github/workflows/rust-security.yml@v1
```

### Workflow `rust.yml`

| Job | Prüft | Befehl (Kurzform) |
|---|---|---|
| `fmt` | Formatierung nach Rust Style Guide | `cargo fmt --all --check` |
| `clippy` | Lints aus der `[lints]`-Tabelle der `Cargo.toml` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `test` | Unit-, Integrations- und Doctests | `cargo test --workspace --all-features` |
| `doc` | Doku baut, keine kaputten Links | `cargo doc --no-deps` mit `RUSTDOCFLAGS=-D warnings` |
| `msrv` | Läuft mit der minimalen Rust-Version (nur wenn `msrv` gesetzt) | `cargo check` |
| `result` | Fasst alle Jobs zusammen, einziger Pflicht-Check im Ruleset | |

Eingaben:

| Input | Standard | Bedeutung |
|---|---|---|
| `toolchain` | `stable` | Toolchain für fmt, clippy, test, doc |
| `msrv` | leer | Minimale Rust-Version, z. B. `1.85`. Leer = kein MSRV-Job |
| `working-directory` | `.` | Ort der `Cargo.toml` bzw. des Workspace-Roots |
| `os` | `["ubuntu-latest"]` | JSON-Liste der Runner für den Test-Job |
| `locked` | `true` | `--locked`: `Cargo.lock` muss eingecheckt und aktuell sein |

### Workflow `rust-security.yml`

Führt `cargo deny check` aus, aufgeteilt in zwei Jobs: `advisories` (RustSec-Datenbank) sowie
`bans licenses sources`. Eingabe: `manifest-path` (Standard `./Cargo.toml`).

## Entscheidungen und Begründungen

So bin ich vorgegangen: erst gefragt, welche Fehler ich vor dem Merge wirklich abfangen will, dann das
einfachste Mittel gewählt, das sie zuverlässig abfängt, und alles andere bewusst weggelassen.

**1. Wiederverwendbare Workflows statt Submodule oder Kopien.**
Ändere ich hier etwas, gilt es in allen Projekten, ohne Updates in jedem Repo. Ein Submodule müsste ich in jedem
Repo einzeln aktualisieren und committen. Die Projekte pinnen auf `@v1`, ein Update hier bricht also nicht
überraschend alle Projekte.

**2. Zwei Workflows statt einem.**
Die Qualitätsprüfung (`rust.yml`) läuft bei jedem Push und PR. Die Sicherheitsprüfung (`rust-security.yml`)
braucht einen anderen Rhythmus: nur bei Änderungen an Dependencies und zusätzlich zeitgesteuert, weil neue
Advisories auch ohne Commits erscheinen. Getrennt kann ich sie unabhängig aufrufen und nur eine als Pflicht-Check
verlangen.

**3. Lints stehen in der `Cargo.toml`, nicht in der CI.**
Die CI übergibt nur `-D warnings`. Welche Lints gelten, steht in `[lints]` im Projekt. So zeigt RustRover schon beim
Tippen exakt dieselben Warnungen, die später die CI meldet. Lokal sind es Warnungen, in der CI Fehler. Die Lints
kopiere ich pro Projekt aus dem Template, weil Cargo sie nicht aus einem anderen Repo laden kann.

**4. Toolchain per `rustup` statt über eine Drittanbieter-Action.**
GitHubs Runner haben `rustup` vorinstalliert. Ein direkter Aufruf spart eine Abhängigkeit, und mit
`cargo +<toolchain>` ist die Version in jedem Schritt explizit, auch wenn ein Projekt eine `rust-toolchain.toml`
hat. Eine Abhängigkeit weniger heißt auch eine Angriffsfläche weniger.

**5. Nur drei externe Actions, alle mit Major-Tag.**
`actions/checkout` (von GitHub), `Swatinem/rust-cache` (Standard für Cargo-Caching) und
`EmbarkStudios/cargo-deny-action`. Ich pinne auf Major-Tags (`@v6`, `@v2`) und nicht auf Commit-SHAs: SHAs wären
sicherer gegen verschobene Tags, aber ohne Automatik schlecht zu pflegen. Dependabot hält die Versionen aktuell
(mit einer Woche Cooldown für frische Releases). Wer es strenger will, kann die Tags mit einem Tool wie `pinact` auf
SHAs umschreiben lassen. Beim Bau waren `actions/checkout@v6` und `rust-cache@v2` sicher vorhanden. Gut möglich, dass
es von checkout schon `v7` gibt, dann schlägt Dependabot es vor.

**6. `--locked` standardmäßig an.**
Die CI soll genau das bauen, was in `Cargo.lock` steht, und scheitern, wenn die Datei veraltet ist. Das macht Builds
reproduzierbar und verhindert, dass ein vergessenes Update unbemerkt bleibt. Die `Cargo.lock` einzuchecken ist
inzwischen auch für Libraries üblich. Wer sie nicht eincheckt, setzt `locked: false`.

**7. Pflicht-Check `result`.**
Ein Ruleset muss die Namen der Pflicht-Checks kennen. Mit Matrix (`test (ubuntu-latest)`) und optionalem MSRV-Job
wären diese Namen je nach Projekt verschieden. `result` läuft immer, schlägt fehl, wenn irgendein Job fehlschlug oder
abgebrochen wurde, und akzeptiert übersprungene Jobs. Das Ruleset braucht damit nur einen Namen: `rust / result`.

**8. Nur Linux als Standard, MSRV nur auf Wunsch.**
Ein Windows- oder macOS-Runner kostet mehr Zeit und Minuten und lohnt sich nur bei plattformabhängigem Code
(`os`-Input). Der MSRV-Job prüft nur mit `cargo check`, denn Tests und Dev-Dependencies dürfen eine neuere Version
brauchen. Er ist optional, weil eine MSRV-Zusage nicht in jedes Projekt gehört.

**9. Der Doku-Job bricht bei Warnungen.**
Kaputte Intra-Doc-Links und andere rustdoc-Warnungen fallen sonst nie auf. Das passt zu den API Guidelines, die
brauchbare Dokumentation verlangen.

**10. cargo-deny statt cargo-audit.**
Ein Werkzeug deckt Advisories, Lizenzen, verbotene Crates und erlaubte Quellen ab. Für reine Advisories gibt es
zusätzlich Dependabot Alerts, die das Setup-Skript im Template einschaltet. Advisories blockieren Pull Requests
**nicht** (`continue-on-error`), weil ein neues Advisory nichts mit der Änderung im PR zu tun haben muss. Im
geplanten Lauf dagegen schlägt der Job fehl, damit mich die Benachrichtigung erreicht.

**11. Sparsame Berechtigungen und Timeouts.**
`permissions: contents: read`, `persist-credentials: false` beim Checkout, keine Secrets, `timeout-minutes` an jedem
Job. Ein hängender Job kostet sonst bis zu sechs Stunden Laufzeit. `concurrency` steht bewusst im aufrufenden
Projekt, weil wiederverwendbare Workflows es nicht für den Aufrufer festlegen können.

**12. Selbsttest mit Fixture-Crate.**
`self-test.yml` ruft die Workflows gegen `fixtures/hello` auf (Linux und Windows, mit MSRV) und prüft die
Workflow-Dateien mit actionlint. Fehler in `rust-ci` fallen so hier auf und nicht erst in einem Projekt.

### Bewusst nicht enthalten

| Was | Warum nicht | Wenn ich es doch brauche |
|---|---|---|
| Matrix stable/beta/nightly | Mehr Laufzeit, selten Erkenntnis | Eigener Job im Projekt oder `toolchain: beta` im Aufruf |
| Coverage, `cargo-nextest`, Miri | Nützlich, aber nicht für jedes Projekt | Eigener Job im Projekt |
| `cargo-semver-checks` | Nur für veröffentlichte Libraries sinnvoll | Eigener Job mit der Action von obi1kenobi |
| Ungenutzte Dependencies (`cargo-machete`) | Liefert gelegentlich Fehlalarme | Lokal bei Bedarf |
| Service-Container (z. B. Postgres für Integrationstests) | Hängt am Projekt, nicht am Rust-Setup | Eigener Workflow im Projekt |
| Release-Automatisierung | Zu projektspezifisch | Eigener Workflow, Release-Notes-Kategorien liegen im Template |

## Lizenz

[MIT](LICENSE)
