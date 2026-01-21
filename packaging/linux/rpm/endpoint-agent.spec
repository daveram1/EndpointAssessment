Name:           endpoint-agent
Version:        0.1.0
Release:        1%{?dist}
Summary:        Endpoint Assessment Agent for system monitoring and compliance checks

License:        MIT
URL:            https://github.com/daveram1/EndpointAssessment
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

Requires:       systemd
Requires(pre):  shadow-utils

%description
Endpoint Assessment Agent collects system information and executes
compliance checks on endpoints, reporting results to a central server.
Supports file checks, process monitoring, port scanning, and custom commands.

%prep
%setup -q

%build
cargo build --release -p agent

%install
rm -rf %{buildroot}

# Binary
install -D -m 755 target/release/agent %{buildroot}%{_bindir}/endpoint-agent

# Systemd service
install -D -m 644 packaging/linux/systemd/endpoint-agent.service %{buildroot}%{_unitdir}/endpoint-agent.service

# Configuration
install -D -m 640 packaging/linux/systemd/agent.conf %{buildroot}%{_sysconfdir}/endpoint-agent/agent.conf

# Log directory
install -d -m 755 %{buildroot}%{_localstatedir}/log/endpoint-agent

%pre
# Create service user
getent passwd endpoint-agent >/dev/null || \
    useradd --system --no-create-home --shell /sbin/nologin endpoint-agent
exit 0

%post
%systemd_post endpoint-agent.service

echo ""
echo "========================================"
echo "Endpoint Assessment Agent installed!"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Edit configuration: sudo nano /etc/endpoint-agent/agent.conf"
echo "2. Set SERVER_URL and AGENT_SECRET"
echo "3. Enable and start: sudo systemctl enable --now endpoint-agent"
echo "4. Check status: sudo systemctl status endpoint-agent"
echo ""

%preun
%systemd_preun endpoint-agent.service

%postun
%systemd_postun_with_restart endpoint-agent.service

if [ $1 -eq 0 ]; then
    # Package removal, not upgrade
    userdel endpoint-agent 2>/dev/null || true
    rm -rf %{_localstatedir}/log/endpoint-agent
fi

%files
%license LICENSE
%doc README.md
%{_bindir}/endpoint-agent
%{_unitdir}/endpoint-agent.service
%dir %{_sysconfdir}/endpoint-agent
%config(noreplace) %attr(640, root, endpoint-agent) %{_sysconfdir}/endpoint-agent/agent.conf
%dir %attr(755, endpoint-agent, endpoint-agent) %{_localstatedir}/log/endpoint-agent

%changelog
* Mon Jan 20 2025 Endpoint Assessment Team <support@example.com> - 0.1.0-1
- Initial package release
