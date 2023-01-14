//! This crate provides a safe binding to `WSAPoll`.
//!
//! On non-windows, this crate is empty.
//!
//! # Minimum Rust version
//!
//! The minimum Rust version required by this crate is 1.34.

#![deny(
    rust_2018_idioms,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_must_use,
    unused_qualifications
)]

#[cfg(windows)]
pub use socket::wsa_poll;

#[cfg(windows)]
mod socket {
    use std::convert::TryInto;
    use std::io;

    use windows_sys::Win32::Networking::WinSock::{
        WSAGetLastError, WSAPoll, SOCKET_ERROR, WSAPOLLFD,
    };

    /// `wsa_poll` waits for one of a set of file descriptors to become ready to
    /// perform I/O.
    ///
    /// This corresponds to calling [`WSAPoll`].
    ///
    /// [`WSAPoll`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsapoll
    pub fn wsa_poll(fd_array: &mut [WSAPOLLFD], timeout: i32) -> io::Result<usize> {
        unsafe {
            let length = fd_array.len().try_into().unwrap();
            let rc = WSAPoll(fd_array.as_mut_ptr(), length, timeout);
            if rc == SOCKET_ERROR {
                return Err(io::Error::from_raw_os_error(WSAGetLastError()));
            };
            Ok(rc.try_into().unwrap())
        }
    }

    #[cfg(test)]
    mod test {
        use std::io::{Result, Write};
        use std::net::{TcpListener, TcpStream};
        use std::os::windows::io::{AsRawSocket, RawSocket};

        use windows_sys::Win32::Networking::WinSock::{POLLIN, POLLOUT, POLLRDNORM, WSAPOLLFD};

        use super::wsa_poll;

        /// Get a pair of connected TcpStreams
        fn get_connection_pair() -> Result<(TcpStream, TcpStream)> {
            let listener = TcpListener::bind("127.0.0.1:0")?;
            let stream1 = TcpStream::connect(listener.local_addr()?)?;
            let stream2 = listener.accept()?.0;

            Ok((stream1, stream2))
        }

        fn poll(socket: RawSocket, events: u16, revents: u16) -> Result<()> {
            let mut sockets = [WSAPOLLFD {
                fd: socket as _,
                events: events as _,
                revents: 0,
            }];
            let count = wsa_poll(&mut sockets, -1)?;
            assert_eq!(count, 1);
            assert_eq!(sockets[0].revents, revents as _);

            Ok(())
        }

        #[test]
        fn test_poll() -> Result<()> {
            let (mut stream1, stream2) = get_connection_pair()?;

            // Check that stream1 is writable
            poll(stream1.as_raw_socket(), POLLOUT, POLLOUT)?;

            // Write something to the stream
            stream1.write_all(b"1")?;

            // stream2 should now be readable and writable
            poll(
                stream2.as_raw_socket(),
                POLLIN | POLLOUT,
                POLLOUT | POLLRDNORM,
            )?;

            Ok(())
        }
    }
}
