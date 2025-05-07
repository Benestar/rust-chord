# A Distributed Hash Table implemented in Rust

Chord is a DHT protocol on a peer-to-peer network architecture. It works as a key-value store and provides two basic operations, namely PUT and GET. The network takes care of important aspects like load balancing, failure handling and recovery.

This implementation is part of the VoidPhone student project and follows the Chord definition by Stoica et al. [1]. The software is provided without warranty of any kind.

[1] I. Stoica, R. Morris, D. Karger, M. F. Kaashoek, and H. Balakrishnan, “Chord: A scalable peer-to-peer lookup service for internet applications,” in Proceedings of the 2001 Conference on Applications, Technologies, Architectures, and Protocols for Computer Communications, ser. SIGCOMM ’01. New York, NY, USA: ACM, 2001, pp. 149–160. [Online]. Available: http://doi.acm.org/10.1145/383059.383071
