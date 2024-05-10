 # curtailing - a toy link shortening service written in Rust

## Just for fun

This is a work in progress and just for fun, to help learn me some Rust. It's
unlikely that this is in a usable state and even when it is, it's not going to
be a good example of either Rust or a link shortener.

## Usage
### Configuration

At the moment the only configuration is through two environment variables, so
either set those in your environment or copy `.env.sample` to `.env` and edit
that; the `curtailing` binary will read it.

#### `CURTAILING_DB_URL` environment variable

At present only an in-memory SQLite database is implemented, so this must be
set to `:memory:`. Any other value will cause the program to exit with a panic.

#### `CURTAILING_LISTEN_ON` environment variable

This defines what IP address and port to listen on and is in the form
`IP:port`. Surround IPv6 addresses with `[]`.

On modern systems you can simultaneously listen on both IPv4 and IPv6 localhost
with `[::]:port`, otherwise you can specify `127.0.0.1:port` or `[::1]:port` if
you want only one or the other. `0.0.0.0:port` would listen on all interfaces.

### Execution
Just launch the `curtailing` binary. There aren't any command line options yet.

For development purposes one or more test links will be inserted into the
in-memory database at each program start. You can see what they are with the
`/api/all` test endpoint described below.

The first `.sql` file in the `migrations/` directory shows the SQL used to
create the initial database, and the first link that is added. You could edit
or add to that if you want other links to always be in there.

### REST API

At the moment the only thing this provides is a REST API. There is not yet any
built-in web app to act as a client. You can try it out with `curl` for
example.  A `curl` example is provided with each endpoint described below.

All API endpoints return an `application/json` response; those expecting data
to be provided BY `POST` require an `application/json` payload. JSON has been
shown in expanded "pretty" format as you might see from `jq` but in reality
will be compressed onto a single line.

#### `POST` to `/api/link` - Create a new short link
Create a new short link mapping in the database.

Since the database is only at present an in-memory SQLite it will be cleared
each time the program is restarted.

##### Payload
```json
{
    "target": "http://example.com/"
}
```

##### Response
During development this will return a response that includes the `uuid` that's
used as a database primary key. That's unnecessary information for the user and
will be removed at some point.

```json
{
    "uuid": "018f4f67-c3e7-7919-b22a-9ab431403ca4",
    "short": "5cf",
    "target": "http://example.com/"
}
```

##### `curl` example
```
curl -v http://localhost:3000/api/link \
    -H "Content-Type: application/json" \
    -d '{"target": "http://example.com/"}'
```

#### `GET` to `/api/link/:short` - Retrieve a link target
##### Payload
The payload is in the URL path as the string after the `/api/link/`.

##### Response
A link object, or an error. As you would expect the error wll be HTTP code 404
if there is no such mapping.

During development this will return a response that includes the `uuid` that's
used as a database primary key. That's unnecessary information for the user and
will be removed at some point.

```json
{
    "uuid": "018f4f67-c3e7-7919-b22a-9ab431403ca4",
    "short": "5cf",
    "target": "http://example.com/"
}
```

##### `curl` example
```
curl -v http://localhost:3000/api/link/5cf
```

#### `GET` to `/api/all` - List off all links in database
This endpoint is for development purposes only. It's not desirable to dump out
the entire contents of the database in a production setting. This endpoint will
go away at some point, or be heavily restricted and put behind authentication.

##### Payload
None.

##### Response
A list of link objects.

During development this will return responses that include the `uuid` that's
used as a database primary key. That's unnecessary information for the user and
will be removed at some point.

```json
[
    {
        "uuid": "018f244b-942b-7007-927b-ace4fadf4a88",
        "short": "6fy",
        "target": "https://mailman.bitfolk.com/mailman/hyperkitty/list/users@mailman.bitfolk.com/message/BV6BHVJN7YL4OYN7C5Y5LRPWJKALPWY6/"
    },
    {
        "uuid": "018f4f67-c3e7-7919-b22a-9ab431403ca4",
        "short": "5cf",
        "target": "http://example.com/"
    }
]
```

##### `curl` example
```
curl -v http://localhost:3000/api/all
```

## Implemented
Things from the roadmap below that have been done.

- [x] Configuration
    - Minimum viable implementation is .env file
- [x] Simple REST API for interacting with an in-memory SQLite DB of short links
    - Minimum viable implementation: rightmost 16 bits of UUIDv7 formatted as Base58
    - Handle database collision for short links since we only have 16 bits to begin with
    - No authentication yet
- [x] Think about DB schema migrations for when the schema inevitably has to change. Maybe also for defining the initial data.
    - An initial migration is now run to populate the DB but little thought has been put into any later migrations that may be needed. Will tackle them as the need arises.

## What we shall laughingly call a roadmap
In rough order of how I'd like to approach things, but things might get juggled
about.

- [ ] Work out how to add tests to this thing
- [ ] Persist DB to disk
- [ ] Add authentication for a single user; only authenticated users should be able to add links
    - User will have to be manually created in the DB or other static config
    - If in static config then probably this demands more complex config file format than .env
- [ ] Actually redirect the shortlinks for non-API usage, i.e. a consumer of this API.
- [ ] Better logging (and tracing?) server-side.
- [ ] Increase number of bits used for shortlinks based on how many links are already in the database. e.g.
    - `<=        65` present = use 16 bits (part of minimum viable implementation)
    - `<=    16,776` present = use 24 bits
    - `<= 4,294,966` present = use 32 bits
    - …etc. to aim to keep ratio about 1 in 1,000 valid short links.
    - *Except that the collision chance for the short links gets too high*, so cap that at 50%, which gives us these bounds:
    - 16 bits = `          301` links before 50% collision chance
    - 24 bits = `        4,822`
    - 32 bits = `       77,162`
    - 40 bits = `    1,234,603`
    - 48 bits = `   19,753,662`
    - 56 bits = `  316,058,596`
    - 62 bits = `2,528,468,770` (only 62 bits available as 2 bits of UUID used for versioning)
    - We'll use the lower of these two limits, so for 16 bits it's going to be <= 65 links, and then as above
- [ ] Multiple users or API tokens
    - Still requiring manual creation of users at this stage
- [ ] Offer alternate formatting? e.g. hyphen-separated English words?
- [ ] Authenticated users can list off their own submitted links by API?
- [ ] Authenticated users can delete their own previously-submitted links by API?
- [ ] Authenticated users can change their own previously-submitted links by API?
- [ ] Self-serve user/API token creation
- [ ] Mark some users/API tokens as admins
- [ ] Allow admin users to delete other users' links from the API
- [ ] Use proper database?

## Inspiration
Large amounts of this came from Jeffrey Chone's Axum demo
https://www.youtube.com/watch?v=XZtlD_m59sM

## License
MIT / Apache
