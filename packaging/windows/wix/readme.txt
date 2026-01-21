Endpoint Assessment Agent
=========================

Installation Complete!

Next Steps:
-----------

1. Configure the agent by setting system environment variables:

   Open PowerShell as Administrator and run:

   [System.Environment]::SetEnvironmentVariable('SERVER_URL', 'http://your-server:8080', 'Machine')
   [System.Environment]::SetEnvironmentVariable('AGENT_SECRET', 'your-agent-secret', 'Machine')

2. Start the service:

   sc start EndpointAgent

3. Check service status:

   sc query EndpointAgent

4. View logs:

   Check Windows Event Viewer > Application logs
   Or: C:\Program Files\Endpoint Assessment Agent\logs\

Troubleshooting:
----------------

- Ensure SERVER_URL and AGENT_SECRET are set correctly
- Verify network connectivity to the server
- Check Windows Firewall allows outbound HTTPS connections

For more information, visit:
https://github.com/daveram1/EndpointAssessment
