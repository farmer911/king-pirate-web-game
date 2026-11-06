# Legacy King Pirate server (BabelTime v298) as a local test oracle. Never deploy or expose this image.
#
# Build context: the clean /home/pirate copy made by stage.sh (no OS files from the infected VM).
# CentOS 7 instead of the original CentOS 6: CentOS 6 glibc segfaults on kernels without vsyscall
# emulation (WSL2, Rancher Desktop). CentOS 7 still ships the old sonames the binaries need
# (libssl.so.10, libcrypto.so.10, libreadline.so.6, libncurses.so.5). The bundled memcached needs
# libevent 1.4, which CentOS 7 lacks, so the distro memcached (same protocol) is used instead.
FROM centos:7

RUN sed -i -e 's/^mirrorlist/#mirrorlist/' \
           -e 's|^#baseurl=http://mirror.centos.org|baseurl=http://vault.centos.org|' /etc/yum.repos.d/CentOS-*.repo \
 && printf '[epel]\nname=EPEL 7 archive\nbaseurl=https://archives.fedoraproject.org/pub/archive/epel/7/x86_64/\ngpgcheck=1\ngpgkey=https://dl.fedoraproject.org/pub/epel/RPM-GPG-KEY-EPEL-7\n' \
      > /etc/yum.repos.d/epel.repo \
 && yum -y install libcurl libxml2 readline ncurses-libs bzip2-libs pcre memcached libmcrypt \
                   net-tools psmisc iproute which \
 && yum clean all \
 # nginx was linked against pcre 7 (libpcre.so.0); CentOS 7 ships the same API as libpcre.so.1.
 && ln -s /usr/lib64/libpcre.so.1 /usr/lib64/libpcre.so.0 \
 && groupadd -g 500 pirate && useradd -u 500 -g 500 -d /home/pirate -M -s /bin/bash pirate

COPY --chown=500:500 pirate /home/pirate
COPY start.sh /start.sh
RUN chmod 755 /start.sh

EXPOSE 8002 80
CMD ["/start.sh"]
