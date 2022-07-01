FROM centos:8

LABEL \
	org.label-schema.schema-version = "1.0" \
	org.label-schema.name = "rpclib" \
	org.label-schema.license = "MIT" \
	org.label-schema.vendor = "Institute for Automation of Complex Power Systems, RWTH Aachen University" \
	org.label-schema.author.name = "Philipp Fensch" \
	org.label-schema.author.email = "philipp.fensch@rwth-aachen.de" \
	org.label-schema.vcs-url = "https://github.com/philipp-fensch/rpc-lib"

RUN cd /etc/yum.repos.d/ && sed -i 's/mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-* && sed -i 's|#baseurl=http://mirror.centos.org|baseurl=http://vault.centos.org|g' /etc/yum.repos.d/CentOS-* && yum update -y

RUN dnf -y update

RUN dnf install -y epel-release dnf-plugins-core && \
    dnf install -y https://rpms.remirepo.net/enterprise/remi-release-8.rpm && \
    dnf config-manager --set-enabled powertools && \
    dnf config-manager --set-enabled remi

RUN dnf install -y make bash git gcc autoconf libtool automake rpcgen \
                   ncurses-devel zlib-devel binutils-devel mesa-libGL-devel \
                   libvdpau-devel mesa-libEGL-devel openssl-devel rpcbind libtirpc-devel\
                   texinfo bison flex python3 which libibverbs libibverbs-devel \
                   libasan cppcheck wget expat-devel xz-devel mesa-libGLU-devel freeglut-devel

ENV LD_LIBRARY_PATH="/usr/local/lib:/usr/local/lib64:${LD_LIBRARY_PATH}"

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.62.0
RUN set -eux; \
    # dpkgArch="$(dpkg --print-architecture)"; \
    dpkgArch="amd64"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='67777ac3bc17277102f2ed73fd5f14c51f4ca5963adadf7f174adf4ebc38747b' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='32a1532f7cef072a667bac53f1a5542c99666c4071af0c9549795bbdb2069ec1' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e50d1deb99048bc5782a0200aa33e4eea70747d49dffdc9d06812fd22a372515' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --component rustfmt clippy --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN dnf -y config-manager --add-repo http://developer.download.nvidia.com/compute/cuda/repos/rhel8/x86_64/cuda-rhel8.repo && \
    dnf --refresh -y install cuda-compiler-11-1 cuda-libraries-devel-11-1 cuda-samples-11-1 cuda-driver-devel-11-1 && \
    ln -s cuda-11.1 /usr/local/cuda && \
    ln -s libcuda.so /usr/local/cuda/targets/x86_64-linux/lib/stubs/libcuda.so.1

ENV PATH="/usr/local/cuda/bin:${PATH}"
ENV LIBRARY_PATH="/usr/local/cuda/targets/x86_64-linux/lib/stubs:$(LIBRARY_PATH}"
ENV LD_LIBRARY_PATH="/usr/local/cuda/lib64:/usr/local/cuda/targets/x86_64-linux/lib/stubs:${LD_LIBRARY_PATH}"
