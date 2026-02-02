import React from 'react';

const AuthBanner = () => {
  const devWallet = "5zFaY3G7e1vWiyX5dbkx7pNQWfH7kTebQ5qfMRZyCieZ"; // <-- This will be injected at build time

  return (
    <div style={{ padding: '2rem', textAlign: 'center', background: '#0f0f1a', color: '#e0e0e0', fontfamily: 'monospace' }}>
      <h2>🔐 Nullblock Dev Access</h2>
      <p>Only the architect’s wallet may enter.</p>
      <p><strong>Current Authorized Address:</strong> <code>{devWallet.slice(0, 16)}...</code></p>
      <p style={{ fontSize: '0.9rem', marginTop: '1rem' }}>
        Need access? Contact @pervySageDev.
        <br />
        For now — the system is under maintenance.
        <br />
        The gate opens only for the one who built it.
      </p>
      <div style={{ border: '1px dashed #444', padding: '1rem', margin: '1rem 0', backgroundColor: '#1a1a2e' }}>
        <small>HTTP Basic Auth required: <code>dev-wallet:{devWallet}</code></small>
      </div>
    </div>
  );
};

export default AuthBanner;