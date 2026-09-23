<img src="mtranslate.svg" alt="mtranslate" width="120">

# mtranslate

Myridia's online translation service. A Rust (axum) backend that translates text between 105 language pairs, stores every translation in MySQL/MariaDB tables, and returns cached results — an automatic translation-memory.

## How it works

1. You call `GET /?s=<source>&t=<target>&v=<text>` (or `POST /` with HTML).
2. The service looks the pair up in the MySQL tables:
   - `a_source_target` links `request_hash` → `(source_id, target_id)`.
   - one table per source language holds the text + hash (e.g. `en`, `th`, `de`).
3. Cache hit → returns the stored translation (`msg: "mtranslated"`).
4. Cache miss → calls Google Translate (via the working `translate.googleapis.com` endpoint), stores the result, links the pair, and returns it (`msg: "gtranslated"`).
5. Input is HTML-sanitized before hashing/storing.

## Quick start

### Prereqs
- Rust toolchain (`cargo build`)
- Docker (for the DB) OR a running MySQL/MariaDB

### 1. Config
On first run the service creates `~/.mtranslate/config.toml` with defaults:

```toml
db_name  = "dbsql1"
db_user  = "dbsql1"
db_pass  = "passpass"
db_host  = "localhost"
db_port  = "3306"
wait_min = 2000
wait_max = 7000
```

`wait_min`/`wait_max` = random throttle in ms applied before outbound Google calls.

### 2. Database (Docker)
```bash
cd dockers
docker compose up -d      # starts MariaDB, seeds from dockers/init/*.sql.gz
```

### 3. Run
```bash
cargo run
```

Server starts on `0.0.0.0:8089`.

## API

### `GET /` — translate text
```
https://translate.myridia.com/?s=en&t=th&v=hello
```
Parameters:
| param | meaning          |
|-------|------------------|
| `s`   | source language  |
| `t`   | target language  |
| `v`   | source text      |

Response:
```json
{
  "target_value": "สวัสดี",
  "target_hash": "f3a1c9d8",
  "target_lang": "th",
  "source_lang": "en",
  "source_hash": "2cf24dba",
  "request_hash": "8a2b3c4d",
  "source_value": "hello",
  "msg": "gtranslated"
}
```

`msg`: `mtranslated` (cache hit), `gtranslated` (new), `db error: ...` (DB unreachable), `missing v,s or t parameter ...` (bad request).

### `POST /` — translate HTML
Same engine, but takes a JSON body and returns HTML with translated text preserving markup:
```bash
curl -X POST http://127.0.0.1:8089/ \
  -H "Content-Type: application/json" \
  -d '{"s":"en","t":"th","html":"<p>hello <b>world</b></p>"}'
```

### `GET /help`
Lists the API and the embedded `codes`.

### `GET /codes`
All supported language codes (JSON).

### `GET /ftl`
Same languages in `xx-YY` (Fluent) form.

### `GET /test`
Smoke-test: returns an empty-shaped response. Use to confirm the server is up.

## Language codes

Supported: `en,th,af,sq,am,ar,hy,az,eu,be,bn,bs,bg,ca,ny,co,hr,cs,da,nl,eo,et,tl,fi,fr,fy,gl,ka,de,el,gu,ht,ha,haw,iw,hi,hu,is,ig,ga,it,ja,jw,kn,kk,km,ko,ku,ky,lo,la,lv,lt,lb,mk,mg,ms,ml,mt,mi,mr,mn,my,ne,no,or,ps,fa,pl,pt,pa,ro,ru,sm,gd,sr,st,sn,sd,si,sk,sl,so,es,su,sw,sv,tg,ta,te,tr,uk,ur,uz,vi,cy,xh,yi,yo,zu` (105 codes). Get the live list from `/codes`.

Note: the internal language tables use the code as table name (e.g. `en`, `th`); per-table names live in `.cargo/config.toml` under `[env].codes` and `[env].ftl` (compile-time embedded).

## Data model (MySQL)

One table per language:
```sql
CREATE TABLE `{}` (
  `id`      int(11) NOT NULL,
  `hash`    varchar(16) NOT NULL DEFAULT '',
  `text`    longtext  NOT NULL DEFAULT '',
  `translit` varchar(255) NOT NULL DEFAULT '',
  PRIMARY KEY (`id`),
  UNIQUE KEY `hash` (`hash`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
```
```sql
ALTER TABLE `{}` MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;
```

Links table:
```sql
CREATE TABLE a_source_target (
  hash varchar(16) NOT NULL,
  source_name varchar(3) NOT NULL,
  target_name varchar(3) NOT NULL,
  source_id int(11) NOT NULL,
  target_id int(11) NOT NULL
);
```
- `{lang}.hash` = last-8 of SHA-256 of the text.
- `a_source_target.hash` = last-8 of SHA-256 of `"{source}_{target}_{text}"`.

Create tables easily via https://textmaker.myridia.com using the statements above.

## Maintenance queries

Delete dangling `a_source_target` links where a source or target row was removed:
```sql
DELETE a
FROM `a_source_target` AS a
LEFT JOIN `sv` AS s ON a.source_id = s.id
LEFT JOIN `da` AS t ON a.target_id = t.id
WHERE a.source_name = 'sv' AND a.target_name = 'da'
  AND (t.id IS NULL OR s.id IS NULL);
```
(drop the join if that table doesn't exist)

Delete rows still containing HTML tags:
```sql
DELETE FROM `{}` WHERE text REGEXP '<[^>]+>';
```

## Hint for Google rate limits
The outbound Google translation uses the `dict-chrome-ex` client endpoint (`translate.googleapis.com/translate_a/single`) which is the most stable for scripted access. If Google starts returning 429/"Sorry", reduce request rate (raise `wait_min`/`wait_max`) or wait before retrying.

## Add remote (Codeberg)
```bash
git remote add codeberg ssh://git@codeberg.org/veto/mtranslate.git
git push codeberg
```