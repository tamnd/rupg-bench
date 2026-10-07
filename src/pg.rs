//! A small client for the PostgreSQL wire protocol, version 3.0.
//!
//! The drivers read their checks through this client, and the YCSB driver sends its statements through it. spec/20 section 20.9 names `libpq`. The harness has no dependencies and no unsafe code, so this module speaks the protocol itself. It has the start of a connection, the simple query protocol, `COPY FROM STDIN`, and prepared statements of the extended protocol with the pipeline of `libpq`.
//!
//! It supports the `trust` and `password` methods of `pg_hba.conf`. The machine scripts set `trust` for the benchmark user on the local host. Values are sent and received as text.

use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;

/// Where the server is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Config {
    /// A host name, or a directory that holds the Unix socket, such as `/var/run/postgresql`.
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) dbname: String,
    pub(crate) password: Option<String>,
}

impl Config {
    /// Parses `key=value` words, as `libpq` does: `host`, `port`, `user`, `dbname` and `password`.
    pub(crate) fn parse(s: &str) -> Result<Config, String> {
        let mut c = Config {
            host: "/var/run/postgresql".to_owned(),
            port: 5432,
            user: "postgres".to_owned(),
            dbname: String::new(),
            password: None,
        };
        for word in s.split_whitespace() {
            let (k, v) = word
                .split_once('=')
                .ok_or(format!("the connection word {word:?} is not key=value"))?;
            match k {
                "host" => c.host = v.to_owned(),
                "port" => {
                    c.port = v.parse().map_err(|_| format!("the port {v:?} is not a number"))?
                }
                "user" => c.user = v.to_owned(),
                "dbname" => c.dbname = v.to_owned(),
                "password" => c.password = Some(v.to_owned()),
                _ => return Err(format!("the connection key {k:?} is not supported")),
            }
        }
        if c.dbname.is_empty() {
            c.dbname = c.user.clone();
        }
        Ok(c)
    }
}

/// An error from the server, or a failure of the connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgError {
    /// The SQLSTATE, empty when the error is not from the server.
    pub(crate) code: String,
    pub(crate) message: String,
}

impl fmt::Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.code.is_empty() {
            f.write_str(&self.message)
        } else {
            write!(f, "{}: {}", self.code, self.message)
        }
    }
}

impl From<std::io::Error> for PgError {
    fn from(e: std::io::Error) -> PgError {
        PgError { code: String::new(), message: format!("connection: {e}") }
    }
}

impl From<PgError> for String {
    fn from(e: PgError) -> String {
        e.to_string()
    }
}

fn client_error(message: impl Into<String>) -> PgError {
    PgError { code: String::new(), message: message.into() }
}

/// The result of one statement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rows {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<String>>>,
    /// The command tag, for example `INSERT 0 1` or `SELECT 3`.
    pub(crate) tag: String,
}

/// The result of one statement of `Conn::queue`: the row count, the field count and the first value of the last row, and the command tag. The other values are not kept.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Done {
    pub(crate) rows: u64,
    pub(crate) fields: u16,
    pub(crate) first: Option<Vec<u8>>,
    pub(crate) tag: String,
}

/// The two kinds of socket.
#[derive(Debug)]
enum Socket {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Socket::Tcp(s) => s.read(buf),
            Socket::Unix(s) => s.read(buf),
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Socket::Tcp(s) => s.write(buf),
            Socket::Unix(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Socket::Tcp(s) => s.flush(),
            Socket::Unix(s) => s.flush(),
        }
    }
}

/// One connection.
#[derive(Debug)]
pub(crate) struct Conn {
    reader: BufReader<Socket>,
    writer: BufWriter<Socket>,
    /// The `server_version` that the server reported at the start.
    pub(crate) server_version: String,
}

impl Conn {
    pub(crate) fn connect(c: &Config) -> Result<Conn, PgError> {
        let (r, w) = if c.host.starts_with('/') {
            let s = UnixStream::connect(format!("{}/.s.PGSQL.{}", c.host, c.port))?;
            (Socket::Unix(s.try_clone()?), Socket::Unix(s))
        } else {
            let s = TcpStream::connect((c.host.as_str(), c.port))?;
            s.set_nodelay(true)?;
            (Socket::Tcp(s.try_clone()?), Socket::Tcp(s))
        };
        let mut conn = Conn {
            reader: BufReader::with_capacity(1 << 16, r),
            writer: BufWriter::with_capacity(1 << 16, w),
            server_version: String::new(),
        };
        let mut body = Vec::new();
        body.extend_from_slice(&196608u32.to_be_bytes());
        for (k, v) in [
            ("user", c.user.as_str()),
            ("database", c.dbname.as_str()),
            ("application_name", "rupg-bench"),
        ] {
            cstr(&mut body, k);
            cstr(&mut body, v);
        }
        body.push(0);
        let len = u32::try_from(body.len() + 4)
            .map_err(|_| client_error("the startup message is too long"))?;
        conn.writer.write_all(&len.to_be_bytes())?;
        conn.writer.write_all(&body)?;
        conn.writer.flush()?;
        loop {
            let (tag, msg) = conn.read_message()?;
            match tag {
                b'R' => {
                    let kind = be32(&msg, 0)?;
                    match kind {
                        0 => {}
                        3 => {
                            let pw = c.password.as_deref().ok_or(client_error(
                                "the server asks for a password and none was given",
                            ))?;
                            let mut m = Vec::new();
                            cstr(&mut m, pw);
                            conn.send(b'p', &m)?;
                            conn.writer.flush()?;
                        }
                        _ => {
                            return Err(client_error(format!(
                                "the server asks for authentication method {kind}. This client supports trust and password. Set trust for the benchmark user in pg_hba.conf"
                            )));
                        }
                    }
                }
                b'S' => {
                    let mut parts = msg.split(|&b| b == 0);
                    if parts.next() == Some(b"server_version") {
                        conn.server_version =
                            String::from_utf8_lossy(parts.next().unwrap_or_default()).into_owned();
                    }
                }
                b'K' | b'N' => {}
                b'E' => return Err(parse_error(&msg)),
                b'Z' => return Ok(conn),
                other => {
                    return Err(client_error(format!(
                        "unexpected message {:?} at the start",
                        other as char
                    )));
                }
            }
        }
    }

    /// Runs one or more statements with the simple query protocol and returns the result of each. An error stops the rest, as in `psql -c`.
    pub(crate) fn simple(&mut self, sql: &str) -> Result<Vec<Rows>, PgError> {
        let mut m = Vec::new();
        cstr(&mut m, sql);
        self.send(b'Q', &m)?;
        self.writer.flush()?;
        let mut out = Vec::new();
        let mut cur = Rows::default();
        let mut err = None;
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'T' => cur.columns = parse_row_description(&msg)?,
                b'D' => cur.rows.push(parse_data_row(&msg)?),
                b'C' => {
                    cur.tag = cstring_at(&msg, 0)?.0;
                    out.push(std::mem::take(&mut cur));
                }
                b'I' => out.push(std::mem::take(&mut cur)),
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => return err.map_or(Ok(out), Err),
                b'N' | b'S' | b'A' => {}
                b'G' | b'H' => return Err(client_error("COPY is not supported by this client")),
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
    }

    /// Runs `COPY ... FROM STDIN` and sends the lines of `input`. When `strip` is set, a line that ends with that byte loses it, as the `|` at the end of each line of a TPC-H `.tbl` file. It returns the command tag, for example `COPY 6001215`.
    pub(crate) fn copy_in(
        &mut self,
        sql: &str,
        input: &mut dyn BufRead,
        strip: Option<u8>,
    ) -> Result<String, PgError> {
        let mut m = Vec::new();
        cstr(&mut m, sql);
        self.send(b'Q', &m)?;
        self.writer.flush()?;
        let mut err = None;
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'G' => break,
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => {
                    return Err(err.unwrap_or(client_error("the statement did not start a COPY")));
                }
                b'N' | b'S' => {}
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
        let mut chunk = Vec::with_capacity(1 << 17);
        let mut line = Vec::new();
        let mut failed = None;
        loop {
            line.clear();
            match input.read_until(b'\n', &mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    failed = Some(e.to_string());
                    break;
                }
            }
            let mut body = line.strip_suffix(b"\n").unwrap_or(&line);
            body = body.strip_suffix(b"\r").unwrap_or(body);
            if let Some(b) = strip {
                body = body.strip_suffix(&[b]).unwrap_or(body);
            }
            chunk.extend_from_slice(body);
            chunk.push(b'\n');
            if chunk.len() >= 1 << 16 {
                self.send(b'd', &chunk)?;
                chunk.clear();
            }
        }
        if let Some(e) = &failed {
            let mut m = Vec::new();
            cstr(&mut m, e);
            self.send(b'f', &m)?;
        } else {
            if !chunk.is_empty() {
                self.send(b'd', &chunk)?;
            }
            self.send(b'c', &[])?;
        }
        self.writer.flush()?;
        let mut tag_text = String::new();
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'C' => tag_text = cstring_at(&msg, 0)?.0,
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => break,
                b'N' | b'S' => {}
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
        match (failed, err) {
            (Some(e), _) => Err(client_error(format!("reading the input: {e}"))),
            (None, Some(e)) => Err(e),
            (None, None) => Ok(tag_text),
        }
    }

    /// Runs one statement with the simple protocol and returns its result.
    pub(crate) fn query(&mut self, sql: &str) -> Result<Rows, PgError> {
        self.simple(sql)?.pop().ok_or(client_error("the statement returned nothing"))
    }

    /// Makes the prepared statement `name` with the extended protocol. The parameters are text and the server finds their types.
    pub(crate) fn prepare(&mut self, name: &str, sql: &str) -> Result<(), PgError> {
        let mut m = Vec::new();
        cstr(&mut m, name);
        cstr(&mut m, sql);
        m.extend_from_slice(&0u16.to_be_bytes());
        self.send(b'P', &m)?;
        self.send(b'S', &[])?;
        self.writer.flush()?;
        let mut err = None;
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'1' | b'N' | b'S' => {}
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => return err.map_or(Ok(()), Err),
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
    }

    /// Puts Bind, Execute and Sync for the prepared statement `name` in the send buffer, with text parameters. Each statement has its own Sync, so it is its own transaction, as with the simple protocol. The statement goes to the server when the buffer is full or at `flush`. More than one statement can wait for its result, as in the pipeline mode of `libpq` with `PQpipelineSync` after each statement.
    pub(crate) fn queue(&mut self, name: &str, params: &[&[u8]]) -> Result<(), PgError> {
        let n = u16::try_from(params.len()).map_err(|_| client_error("too many parameters"))?;
        let mut m = Vec::with_capacity(32 + params.iter().map(|p| p.len() + 4).sum::<usize>());
        // The unnamed portal, the statement, no format codes (all text), the parameters, no result format codes (all text).
        m.push(0);
        cstr(&mut m, name);
        m.extend_from_slice(&0u16.to_be_bytes());
        m.extend_from_slice(&n.to_be_bytes());
        for p in params {
            let len =
                i32::try_from(p.len()).map_err(|_| client_error("a parameter is too long"))?;
            m.extend_from_slice(&len.to_be_bytes());
            m.extend_from_slice(p);
        }
        m.extend_from_slice(&0u16.to_be_bytes());
        self.send(b'B', &m)?;
        // The unnamed portal, all rows.
        self.send(b'E', &[0, 0, 0, 0, 0])?;
        self.send(b'S', &[])?;
        Ok(())
    }

    /// Sends the statements in the send buffer.
    pub(crate) fn flush(&mut self) -> Result<(), PgError> {
        self.writer.flush()?;
        Ok(())
    }

    /// Reads the result of the oldest statement of `queue` that has no result yet. The outer error is a failure of the connection. The inner error is an error that the server sent for the statement.
    pub(crate) fn next_done(&mut self) -> Result<Result<Done, PgError>, PgError> {
        let mut done = Done::default();
        let mut err = None;
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'2' | b'n' | b'N' | b'S' => {}
                b'D' => {
                    done.rows += 1;
                    done.fields = be16(&msg, 0)?;
                    done.first = first_value(&msg)?;
                }
                b'C' => done.tag = cstring_at(&msg, 0)?.0,
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => return Ok(err.map_or(Ok(done), Err)),
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
    }

    /// As `next_done`, but it keeps the values of the rows as text. The column names are empty, because `queue` does not ask for a description.
    pub(crate) fn next_rows(&mut self) -> Result<Result<Rows, PgError>, PgError> {
        let mut rows = Rows::default();
        let mut err = None;
        loop {
            let (tag, msg) = self.read_message()?;
            match tag {
                b'2' | b'n' | b'N' | b'S' => {}
                b'D' => rows.rows.push(parse_data_row(&msg)?),
                b'C' => rows.tag = cstring_at(&msg, 0)?.0,
                b'E' => err = Some(parse_error(&msg)),
                b'Z' => return Ok(err.map_or(Ok(rows), Err)),
                other => {
                    return Err(client_error(format!("unexpected message {:?}", other as char)));
                }
            }
        }
    }

    fn send(&mut self, tag: u8, body: &[u8]) -> Result<(), PgError> {
        let len =
            u32::try_from(body.len() + 4).map_err(|_| client_error("a message is too long"))?;
        self.writer.write_all(&[tag])?;
        self.writer.write_all(&len.to_be_bytes())?;
        self.writer.write_all(body)?;
        Ok(())
    }

    fn read_message(&mut self) -> Result<(u8, Vec<u8>), PgError> {
        let mut head = [0u8; 5];
        self.reader.read_exact(&mut head)?;
        let len = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
        if len < 4 {
            return Err(client_error("a message with a length below 4"));
        }
        let mut body = vec![0u8; len - 4];
        self.reader.read_exact(&mut body)?;
        Ok((head[0], body))
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        // Terminate. A failure here does not matter, the socket closes anyway.
        let _ = self.send(b'X', &[]);
        let _ = self.writer.flush();
    }
}

fn cstr(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    out.push(0);
}

fn be32(b: &[u8], at: usize) -> Result<u32, PgError> {
    b.get(at..at + 4)
        .map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or(client_error("a short message"))
}

fn be16(b: &[u8], at: usize) -> Result<u16, PgError> {
    b.get(at..at + 2)
        .map(|s| u16::from_be_bytes([s[0], s[1]]))
        .ok_or(client_error("a short message"))
}

/// The string at `at` and the offset after its terminating zero.
fn cstring_at(b: &[u8], at: usize) -> Result<(String, usize), PgError> {
    let rest = b.get(at..).ok_or(client_error("a short message"))?;
    let end = rest.iter().position(|&c| c == 0).ok_or(client_error("a string with no end"))?;
    Ok((String::from_utf8_lossy(&rest[..end]).into_owned(), at + end + 1))
}

fn parse_row_description(b: &[u8]) -> Result<Vec<String>, PgError> {
    let n = be16(b, 0)?;
    let mut at = 2;
    let mut out = Vec::with_capacity(n.into());
    for _ in 0..n {
        let (name, next) = cstring_at(b, at)?;
        out.push(name);
        // Table oid, column number, type oid, type size, type modifier, format code.
        at = next + 18;
    }
    Ok(out)
}

fn parse_data_row(b: &[u8]) -> Result<Vec<Option<String>>, PgError> {
    let n = be16(b, 0)?;
    let mut at = 2;
    let mut out = Vec::with_capacity(n.into());
    for _ in 0..n {
        let len = be32(b, at)? as i32;
        at += 4;
        if len < 0 {
            out.push(None);
        } else {
            let end = at + len as usize;
            let v = b.get(at..end).ok_or(client_error("a short data row"))?;
            out.push(Some(String::from_utf8_lossy(v).into_owned()));
            at = end;
        }
    }
    Ok(out)
}

/// The first value of a data row, None for NULL or for a row with no values.
fn first_value(b: &[u8]) -> Result<Option<Vec<u8>>, PgError> {
    if be16(b, 0)? == 0 {
        return Ok(None);
    }
    let len = be32(b, 2)? as i32;
    if len < 0 {
        return Ok(None);
    }
    let v = b.get(6..6 + len as usize).ok_or(client_error("a short data row"))?;
    Ok(Some(v.to_vec()))
}

fn parse_error(b: &[u8]) -> PgError {
    let mut code = String::new();
    let mut message = String::new();
    let mut at = 0;
    while at < b.len() && b[at] != 0 {
        let field = b[at];
        let Ok((value, next)) = cstring_at(b, at + 1) else { break };
        match field {
            b'C' => code = value,
            b'M' => message = value,
            _ => {}
        }
        at = next;
    }
    PgError { code, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_words() {
        let c = Config::parse("host=127.0.0.1 port=5433 user=bench").unwrap();
        assert_eq!(
            (c.host.as_str(), c.port, c.user.as_str(), c.dbname.as_str()),
            ("127.0.0.1", 5433, "bench", "bench")
        );
        let d = Config::parse("").unwrap();
        assert_eq!(
            (d.host.as_str(), d.port, d.dbname.as_str()),
            ("/var/run/postgresql", 5432, "postgres")
        );
        assert!(Config::parse("sslmode=require").is_err());
        assert!(Config::parse("port=x").is_err());
    }

    #[test]
    fn data_row_with_a_null() {
        let mut b = Vec::new();
        b.extend_from_slice(&2u16.to_be_bytes());
        b.extend_from_slice(&3i32.to_be_bytes());
        b.extend_from_slice(b"abc");
        b.extend_from_slice(&(-1i32).to_be_bytes());
        assert_eq!(parse_data_row(&b).unwrap(), vec![Some("abc".to_owned()), None]);
        assert!(parse_data_row(&b[..6]).is_err());
    }

    #[test]
    fn row_description_names() {
        let mut b = Vec::new();
        b.extend_from_slice(&2u16.to_be_bytes());
        for name in ["aid", "abalance"] {
            cstr(&mut b, name);
            b.extend_from_slice(&[0; 18]);
        }
        assert_eq!(parse_row_description(&b).unwrap(), vec!["aid", "abalance"]);
    }

    #[test]
    fn error_fields() {
        let mut b = Vec::new();
        for (f, v) in [(b'S', "ERROR"), (b'C', "40001"), (b'M', "could not serialize access")] {
            b.push(f);
            cstr(&mut b, v);
        }
        b.push(0);
        let e = parse_error(&b);
        assert_eq!(e.to_string(), "40001: could not serialize access");
    }

    #[test]
    fn first_value_of_a_row() {
        let mut b = Vec::new();
        b.extend_from_slice(&2u16.to_be_bytes());
        b.extend_from_slice(&5i32.to_be_bytes());
        b.extend_from_slice(b"user1");
        b.extend_from_slice(&(-1i32).to_be_bytes());
        assert_eq!(first_value(&b).unwrap(), Some(b"user1".to_vec()));
        assert_eq!(first_value(&0u16.to_be_bytes()).unwrap(), None);
        assert!(first_value(&b[..7]).is_err());
    }
}
