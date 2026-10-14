//! A stream whose TLS is ours: a captured fingerprint's ClientHello, our own
//! handshake, and application data through our own record layer.
//!
//! The adapter is thin on purpose. `core_tls::client::Connection` works on
//! bytes -- plaintext in, one record out -- so all this has to do is move bytes
//! between the socket and those two calls without ever blocking inside a poll.
//! The blocking parts of the TLS live in core-tls, where they can be tested
//! against a real server; here there is only a buffer and two state flags.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use core_tls::client::{Connection, Record};

/// How much the raw buffer grows by when it needs room. A record is at most
/// 2^14 + 256 bytes with its tag, so one read fills several of them on any socket
/// that is not pathological.
const CHUNK: usize = 64 * 1024;

/// TLS over `S`, established by us rather than by a library.
pub struct FingerprintStream<S> {
    inner: S,
    connection: Connection,
    /// transport bytes, and where the opened plaintext of the record at the front
    /// of them starts and ends
    raw: Vec<u8>,
    /// how far into `raw` the bytes already handed over run. A 64 KiB read holds
    /// four records, and shifting what is left down after each one moves three
    /// times the payload it just finished reading: a second pass over every byte
    /// on the hottest path in the tree. The cursor costs nothing, and the buffer
    /// is compacted only when it is actually full.
    rpos: usize,
    /// `raw[plain_at..plain_end]` has been opened and not handed over yet, and
    /// the record it came from runs to `plain_end` no further: `raw[..record_end]`
    /// is what a caller drops once it is done with the plaintext
    plain_at: usize,
    plain_end: usize,
    record_end: usize,
    /// the sealed record waiting to go out, kept from record to record:
    /// sealing used to allocate a fresh Vec per record, which on the hot
    /// path is steady churn for bytes that land in the same 16 KiB slot
    wbuf: Vec<u8>,
    /// how far into `wbuf` the socket has taken; `None` between records
    out: Option<usize>,
    /// Plaintext length of the sealed record, independent of later input growth.
    pending_plaintext: usize,
    eof: bool,
    direct_read: bool,
}

impl<S> FingerprintStream<S> {
    /// Wrap a transport whose handshake is already finished.
    pub fn new(inner: S, connection: Connection) -> Self {
        FingerprintStream {
            inner,
            connection,
            raw: Vec::new(),
            rpos: 0,
            plain_at: 0,
            plain_end: 0,
            record_end: 0,
            wbuf: Vec::new(),
            out: None,
            pending_plaintext: 0,
            eof: false,
            direct_read: false,
        }
    }

    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub(crate) fn enable_direct_read(&mut self) {
        self.direct_read = true;
        self.release_direct_read_buffer();
    }

    fn release_direct_read_buffer(&mut self) {
        if self.direct_read && self.plain_at == self.plain_end {
            release_drained_raw(&mut self.raw, &mut self.rpos);
        }
    }
}

fn release_drained_raw(raw: &mut Vec<u8>, rpos: &mut usize) {
    if *rpos == raw.len() {
        *raw = Vec::new();
        *rpos = 0;
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for FingerprintStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let me = self.get_mut();
        loop {
            // plaintext the reader's buffer could not take whole
            if me.plain_at < me.plain_end {
                let take = (me.plain_end - me.plain_at).min(buf.remaining());
                buf.put_slice(&me.raw[me.plain_at..me.plain_at + take]);
                me.plain_at += take;
                if me.plain_at == me.plain_end {
                    // the record is out of the caller's hands: its transport bytes
                    // are behind the cursor, and the buffer keeps its room
                    me.rpos = me.record_end;
                    me.plain_at = 0;
                    me.plain_end = 0;
                    me.record_end = 0;
                    me.release_direct_read_buffer();
                }
                return Poll::Ready(Ok(()));
            }
            if me.direct_read {
                // The last decrypted record may share a socket read with the
                // first raw Vision records. Drain those bytes before the socket.
                let take = (me.raw.len() - me.rpos).min(buf.remaining());
                if take > 0 {
                    buf.put_slice(&me.raw[me.rpos..me.rpos + take]);
                    me.rpos += take;
                    me.release_direct_read_buffer();
                    return Poll::Ready(Ok(()));
                }
                me.release_direct_read_buffer();
                return Pin::new(&mut me.inner).poll_read(cx, buf);
            }
            if me.eof || me.connection.is_closed() {
                return Poll::Ready(Ok(()));
            }
            // Whatever we already hold comes first, and only then the socket.
            // The order matters: a socket hands us half a record all the time, so
            // "five bytes buffered" says nothing about whether a record is whole.
            // Asking the socket only when the buffer was *short* meant a split
            // record left us spinning over the same bytes forever, never reading
            // the rest of it -- which is what a REALITY server's post-handshake
            // ticket is.
            match me.connection.read_record_in_place(&mut me.raw[me.rpos..]) {
                Ok(Some(Record::Data {
                    start,
                    end,
                    record_end,
                })) => {
                    me.plain_at = me.rpos + start;
                    me.plain_end = me.rpos + end;
                    me.record_end = me.rpos + record_end;
                    continue;
                }
                Ok(Some(Record::Skip { record_end })) => {
                    me.rpos += record_end;
                    continue;
                }
                Ok(None) => {}
                Err(e) => return Poll::Ready(Err(e)),
            }
            // Read straight into the buffer's spare room: a zeroed stack array per
            // read is 18 KiB of memset plus a copy out of it, on every read. The
            // cursor is folded away here and nowhere else, because this is the one
            // place that needs the room.
            if me.raw.len() == me.raw.capacity() {
                if me.rpos > 0 {
                    let left = me.raw.len() - me.rpos;
                    me.raw.copy_within(me.rpos.., 0);
                    me.raw.truncate(left);
                    me.rpos = 0;
                }
                if me.raw.len() == me.raw.capacity() {
                    me.raw.reserve(CHUNK);
                }
            }
            let have = me.raw.len();
            let got = {
                let mut read = ReadBuf::uninit(me.raw.spare_capacity_mut());
                match Pin::new(&mut me.inner).poll_read(cx, &mut read) {
                    Poll::Ready(Ok(())) => read.filled().len(),
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => return Poll::Pending,
                }
            };
            if got == 0 {
                me.eof = true;
                return Poll::Ready(Ok(()));
            }
            // SAFETY: the reader initialised exactly `got` bytes of the spare
            // capacity, starting at `have`
            unsafe { me.raw.set_len(have + got) };
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for FingerprintStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let me = self.get_mut();
        // A writer that reports writing a 22-byte tag has not written is lying
        // about its input, and every caller that respects the contract -- tokio's
        // `write_all` above all -- slices its buffer by the number it is given.
        // An empty write is nothing to seal: reporting zero for it is the honest
        // answer, and it keeps a caller from sealing an empty record.
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        // one record per call, so the sequence number moves with each record
        // rather than with each call. A caller with more than a record's worth
        // writes it in pieces: `write_record` refuses more, and the number handed
        // back has to be what really left.
        let take = buf.len().min(core_tls::record::MAX_FRAGMENT);
        loop {
            let Some(pos) = me.out else {
                me.wbuf.clear();
                if let Err(e) = me.connection.write_record_into(&buf[..take], &mut me.wbuf) {
                    return Poll::Ready(Err(e));
                }
                me.out = Some(0);
                me.pending_plaintext = take;
                continue;
            };
            match Pin::new(&mut me.inner).poll_write(cx, &me.wbuf[pos..]) {
                Poll::Ready(Ok(n)) => {
                    let pos = pos + n;
                    if pos == me.wbuf.len() {
                        me.out = None;
                        let written = std::mem::take(&mut me.pending_plaintext);
                        return Poll::Ready(Ok(written));
                    }
                    me.out = Some(pos);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // poll_write only returns Ready once a record is whole, so there is
        // never a part-written record to flush around
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let me = self.get_mut();
        me.eof = true;
        Pin::new(&mut me.inner).poll_shutdown(cx)
    }
}

/// One read has to be able to hold a whole record, or every record costs two
/// syscalls: 2^14 of plaintext, a type byte, a 16-byte tag, a 5-byte header.
/// A compile-time check, because a runtime one would only ever be a comment.
const _: () = assert!(CHUNK > 16384 + 1 + 16 + 5);

#[cfg(test)]
mod memory_tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    struct PauseWriteOnce {
        inner: tokio::io::DuplexStream,
        pause: bool,
    }

    impl AsyncRead for PauseWriteOnce {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for PauseWriteOnce {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            if self.pause {
                self.pause = false;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Pin::new(&mut self.inner).poll_write(cx, buf)
        }
        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_flush(cx)
        }
        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.inner).poll_shutdown(cx)
        }
    }

    #[tokio::test]
    async fn direct_read_preserves_pending_plaintext_and_tail_without_disabling_tls_write() {
        use core_tls::client::{Config, Handshake};
        use core_tls::rustls::{RootCertStore, ServerConfig};
        use core_tls::verify::WebpkiVerifier;
        use std::sync::Arc;

        core_tls::install_default_provider();
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/certs");
        let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(
            std::fs::File::open(dir.join("localhost.crt")).unwrap(),
        ))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(
            std::fs::File::open(dir.join("localhost.key")).unwrap(),
        ))
        .unwrap()
        .unwrap();
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs.clone(), key)
            .unwrap();
        let mut roots = RootCertStore::empty();
        roots.add(certs[0].clone()).unwrap();
        let verifier = WebpkiVerifier::new(Arc::new(roots)).unwrap();
        let template = core_tls::fingerprints::by_name("chrome").unwrap();
        let (mut client, peer) = tokio::io::duplex(CHUNK);
        let server = tokio::spawn(async move {
            let mut tls = tokio_rustls::TlsAcceptor::from(Arc::new(server_config))
                .accept(peer)
                .await
                .unwrap();
            let mut request = [0; 2];
            tls.read_exact(&mut request).await.unwrap();
            assert_eq!(&request, b"hi");
        });
        let mut handshake = Handshake::new(Config {
            template: &template,
            server_name: "localhost",
            alpn: &[],
            verifier: &verifier,
            reality: None,
            fragment: false,
            cipher_suites: Vec::new(),
        })
        .unwrap();
        handshake.run(&mut client).await.unwrap();
        let connection = Connection::new(&mut handshake).unwrap();
        let mut stream = FingerprintStream::new(
            PauseWriteOnce {
                inner: client,
                pause: true,
            },
            connection,
        );
        // State at the transition: decrypted plaintext followed by raw peer bytes.
        stream.raw = Vec::with_capacity(CHUNK);
        stream.raw.extend_from_slice(b"plainRAW");
        stream.plain_at = 0;
        stream.plain_end = 5;
        stream.record_end = 5;
        stream.enable_direct_read();
        assert_eq!(stream.raw.capacity(), CHUNK);
        let mut plaintext = [0; 5];
        stream.read_exact(&mut plaintext).await.unwrap();
        assert_eq!(&plaintext, b"plain");
        assert_eq!(stream.raw.capacity(), CHUNK);
        let mut tail = [0; 3];
        stream.read_exact(&mut tail).await.unwrap();
        assert_eq!(&tail, b"RAW");
        assert_eq!(stream.raw.capacity(), 0);
        let mut cx = Context::from_waker(std::task::Waker::noop());
        assert!(
            Pin::new(&mut stream)
                .poll_write(&mut cx, b"hi")
                .is_pending()
        );
        // The relay may append input while its writer is pending. Only the
        // original record's plaintext was consumed, even if this slice grew.
        let written = std::future::poll_fn(|cx| Pin::new(&mut stream).poll_write(cx, b"hiEXTRA"))
            .await
            .unwrap();
        assert_eq!(
            written, 2,
            "must report the pending record's original length"
        );
        server.await.unwrap();
    }

    #[test]
    fn direct_read_keeps_unread_tail_and_releases_only_when_drained() {
        let mut raw = Vec::with_capacity(CHUNK);
        raw.extend_from_slice(b"encryptedraw tail");
        let mut rpos = 9;
        release_drained_raw(&mut raw, &mut rpos);
        assert_eq!(&raw[rpos..], b"raw tail");
        assert_eq!(raw.capacity(), CHUNK);
        rpos += 3;
        release_drained_raw(&mut raw, &mut rpos);
        assert_eq!(&raw[rpos..], b" tail");
        rpos = raw.len();
        release_drained_raw(&mut raw, &mut rpos);
        assert_eq!(raw.capacity(), 0);
        assert_eq!(rpos, 0);
        release_drained_raw(&mut raw, &mut rpos);
        assert_eq!(raw.capacity(), 0);
    }
}
