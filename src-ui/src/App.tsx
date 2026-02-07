import { useState } from 'react';
import {
  walletGenerate,
  walletLoad,
  balanceGet,
  walletHdGenerate,
  walletHdLoad,
  deriveAddress,
  deriveMany,
  WalletInfo,
  BalanceInfo,
  HDWalletInfo,
  DerivedAddressInfo,
} from './services/api';

type TabType = 'wallet' | 'compose' | 'hdwallet' | 'status';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('wallet');
  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [hdWallet, setHdWallet] = useState<HDWalletInfo | null>(null);
  const [balance, setBalance] = useState<BalanceInfo | null>(null);
  const [privateKey, setPrivateKey] = useState('');
  const [hdSeed, setHdSeed] = useState('');
  const [rpcUrl, setRpcUrl] = useState('https://api-tn10.kaspa.org');
  const [status, setStatus] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [derivedAddresses, setDerivedAddresses] = useState<DerivedAddressInfo[]>([]);
  const [deriveIndex, setDeriveIndex] = useState(0);
  const [deriveCount, setDeriveCount] = useState(5);

  const showStatus = (type: 'success' | 'error' | 'info', text: string) => {
    setStatus({ type, text });
    setTimeout(() => setStatus(null), 8000);
  };

  const handleGenerateWallet = async () => {
    setIsLoading(true);
    try {
      const newWallet = await walletGenerate();
      setWallet(newWallet);
      showStatus('success', 'Wallet generated successfully!');
      setActiveTab('compose');
    } catch (error) {
      showStatus('error', `Failed to generate wallet: ${error}`);
    }
    setIsLoading(false);
  };

  const handleLoadWallet = async () => {
    if (!privateKey.trim()) {
      showStatus('error', 'Please enter a private key');
      return;
    }
    setIsLoading(true);
    try {
      const loadedWallet = await walletLoad(privateKey.trim());
      setWallet(loadedWallet);
      showStatus('success', 'Wallet loaded successfully!');
      setActiveTab('compose');
    } catch (error) {
      showStatus('error', `Failed to load wallet: ${error}`);
    }
    setIsLoading(false);
  };

  const handleHdGenerate = async () => {
    setIsLoading(true);
    try {
      const seed = await walletHdGenerate();
      setHdWallet(seed);
      setHdSeed(seed.seed);
      showStatus('success', 'HD wallet generated! Save your seed phrase.');
    } catch (error) {
      showStatus('error', `Failed to generate HD wallet: ${error}`);
    }
    setIsLoading(false);
  };

  const handleHdLoad = async () => {
    if (!hdSeed.trim()) {
      showStatus('error', 'Please enter a seed phrase');
      return;
    }
    setIsLoading(true);
    try {
      const loaded = await walletHdLoad(hdSeed.trim());
      setHdWallet(loaded);
      showStatus('success', 'HD wallet loaded successfully!');
      setActiveTab('hdwallet');
    } catch (error) {
      showStatus('error', `Failed to load HD wallet: ${error}`);
    }
    setIsLoading(false);
  };

  const handleDeriveAddress = async () => {
    if (!hdWallet) return;
    setIsLoading(true);
    try {
      const addr = await deriveAddress(hdWallet.seed, deriveIndex, false);
      setDerivedAddresses([addr]);
      showStatus('success', `Address derived at index ${deriveIndex}`);
    } catch (error) {
      showStatus('error', `Failed to derive address: ${error}`);
    }
    setIsLoading(false);
  };

  const handleDeriveMany = async () => {
    if (!hdWallet) return;
    setIsLoading(true);
    try {
      const addrs = await deriveMany(hdWallet.seed, deriveCount);
      setDerivedAddresses(addrs);
      showStatus('success', `Derived ${deriveCount} addresses`);
    } catch (error) {
      showStatus('error', `Failed to derive addresses: ${error}`);
    }
    setIsLoading(false);
  };

  const handleCheckBalance = async () => {
    if (!wallet && !hdWallet) return;
    const address = wallet?.address || hdWallet?.address;
    if (!address) return;
    setIsLoading(true);
    try {
      const bal = await balanceGet(address, rpcUrl);
      setBalance(bal);
      showStatus('success', `Balance: ${(bal.balance / 100000000).toFixed(8)} KAS`);
    } catch (error) {
      showStatus('error', `Failed to get balance: ${error}`);
    }
    setIsLoading(false);
  };

  const formatKaspa = (sompi: number): string => {
    return (sompi / 100000000).toFixed(8);
  };

  return (
    <div className="container">
      <header className="header">
        <h1>KaspaGraffiti</h1>
        <p>Post messages to the Kaspa blockchain (testnet-10)</p>
      </header>

      {status && (
        <div className={`status status-${status.type}`}>
          {status.text}
        </div>
      )}

      <div className="tab-container">
        <button
          className={`tab ${activeTab === 'wallet' ? 'active' : ''}`}
          onClick={() => setActiveTab('wallet')}
        >
          Wallet
        </button>
        <button
          className={`tab ${activeTab === 'compose' ? 'active' : ''}`}
          onClick={() => setActiveTab('compose')}
          disabled={!wallet}
        >
          Compose
        </button>
        <button
          className={`tab ${activeTab === 'hdwallet' ? 'active' : ''}`}
          onClick={() => setActiveTab('hdwallet')}
        >
          HD Wallet
        </button>
        <button
          className={`tab ${activeTab === 'status' ? 'active' : ''}`}
          onClick={() => setActiveTab('status')}
          disabled={!wallet && !hdWallet}
        >
          Status
        </button>
      </div>

      {activeTab === 'wallet' && (
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Wallet</h2>
          </div>

          <div className="btn-group" style={{ marginBottom: 24 }}>
            <button
              className="btn btn-primary"
              onClick={handleGenerateWallet}
              disabled={isLoading}
            >
              {isLoading ? 'Generating...' : 'Generate New Wallet'}
            </button>
          </div>

          <div className="form-group">
            <label>Or Load Existing Wallet</label>
            <input
              type="text"
              placeholder="Enter private key (hex)"
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
            />
          </div>

          <div className="btn-group">
            <button
              className="btn btn-secondary"
              onClick={handleLoadWallet}
              disabled={isLoading || !privateKey.trim()}
            >
              {isLoading ? 'Loading...' : 'Load Wallet'}
            </button>
          </div>

          {wallet && (
            <div className="wallet-info" style={{ marginTop: 24 }}>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Address</span>
                <span className="wallet-info-value">{wallet.address}</span>
              </div>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Public Key</span>
                <span className="wallet-info-value">{wallet.public_key}</span>
              </div>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Private Key</span>
                <span className="wallet-info-value" style={{ color: '#f56565' }}>
                  {wallet.private_key}
                </span>
              </div>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Network</span>
                <span className="wallet-info-value">{wallet.network}</span>
              </div>
            </div>
          )}
        </div>
      )}

      {activeTab === 'compose' && wallet && (
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Compose Message</h2>
          </div>
          <p style={{ color: 'var(--text-muted)', marginBottom: 16 }}>
            Graffiti feature coming soon - wallet and balance features are working!
          </p>

          <div className="form-group">
            <label>Your Address</label>
            <input type="text" value={wallet.address} disabled />
          </div>

          <div className="form-group">
            <label>RPC URL (testnet-10)</label>
            <input
              type="text"
              value={rpcUrl}
              onChange={(e) => setRpcUrl(e.target.value)}
              placeholder="https://api-tn10.kaspa.org"
            />
          </div>

          <div className="btn-group">
            <button
              className="btn btn-primary"
              onClick={handleCheckBalance}
              disabled={isLoading}
            >
              {isLoading ? 'Loading...' : 'Check Balance'}
            </button>
          </div>

          {balance && (
            <div style={{ marginTop: 16 }}>
              <span style={{ color: 'var(--text-muted)' }}>Balance: </span>
              <span className="balance">{formatKaspa(balance.balance)} KAS</span>
            </div>
          )}
        </div>
      )}

      {activeTab === 'hdwallet' && (
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">HD Wallet</h2>
          </div>

          <div className="btn-group" style={{ marginBottom: 24 }}>
            <button
              className="btn btn-primary"
              onClick={handleHdGenerate}
              disabled={isLoading}
            >
              {isLoading ? 'Generating...' : 'Generate New HD Wallet'}
            </button>
          </div>

          <div className="form-group">
            <label>Or Load HD Wallet from Seed</label>
            <textarea
              placeholder="Enter your 64-character seed hex"
              value={hdSeed}
              onChange={(e) => setHdSeed(e.target.value)}
              rows={3}
            />
          </div>

          <div className="btn-group">
            <button
              className="btn btn-secondary"
              onClick={handleHdLoad}
              disabled={isLoading || !hdSeed.trim()}
            >
              {isLoading ? 'Loading...' : 'Load HD Wallet'}
            </button>
          </div>

          {hdWallet && (
            <div className="wallet-info" style={{ marginTop: 24 }}>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Seed</span>
                <span className="wallet-info-value" style={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
                  {hdWallet.seed}
                </span>
              </div>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Master Address</span>
                <span className="wallet-info-value">{hdWallet.address}</span>
              </div>
              <div className="wallet-info-item">
                <span className="wallet-info-label">Network</span>
                <span className="wallet-info-value">{hdWallet.network}</span>
              </div>
            </div>
          )}

          {hdWallet && (
            <>
              <hr style={{ margin: '24px 0', border: 'none', borderTop: '1px solid var(--border)' }} />
              
              <h3 style={{ marginBottom: 16 }}>Derive Addresses</h3>
              
              <div style={{ display: 'flex', gap: 16, marginBottom: 16 }}>
                <div className="form-group" style={{ flex: 1 }}>
                  <label>Index</label>
                  <input
                    type="number"
                    value={deriveIndex}
                    onChange={(e) => setDeriveIndex(Number(e.target.value))}
                    min={0}
                  />
                </div>
                <div className="form-group" style={{ flex: 1 }}>
                  <label>Count</label>
                  <input
                    type="number"
                    value={deriveCount}
                    onChange={(e) => setDeriveCount(Number(e.target.value))}
                    min={1}
                    max={100}
                  />
                </div>
              </div>

              <div className="btn-group" style={{ marginBottom: 16 }}>
                <button
                  className="btn btn-secondary"
                  onClick={handleDeriveAddress}
                  disabled={isLoading}
                >
                  Derive Single Address
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={handleDeriveMany}
                  disabled={isLoading}
                >
                  Derive {deriveCount} Addresses
                </button>
              </div>

              {derivedAddresses.length > 0 && (
                <div className="derived-addresses">
                  {derivedAddresses.map((addr, idx) => (
                    <div key={idx} className="derived-address-item">
                      <div><strong>Index:</strong> {addr.index}</div>
                      <div style={{ fontFamily: 'monospace', fontSize: '0.7rem', wordBreak: 'break-all' }}>
                        {addr.address}
                      </div>
                      <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                        Private: {addr.private_key.substring(0, 16)}...
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      )}

      {activeTab === 'status' && (wallet || hdWallet) && (
        <div className="card">
          <div className="card-header">
            <h2 className="card-title">Network Status</h2>
          </div>

          <div className="wallet-info">
            <div className="wallet-info-item">
              <span className="wallet-info-label">Address</span>
              <span className="wallet-info-value">{wallet?.address || hdWallet?.address}</span>
            </div>
            <div className="wallet-info-item">
              <span className="wallet-info-label">Balance</span>
              <span className="wallet-info-value balance">
                {balance ? formatKaspa(balance.balance) : 'Loading...'} KAS
              </span>
            </div>
            <div className="wallet-info-item">
              <span className="wallet-info-label">Network</span>
              <span className="wallet-info-value">testnet-10</span>
            </div>
            <div className="wallet-info-item">
              <span className="wallet-info-label">RPC</span>
              <span className="wallet-info-value">{rpcUrl}</span>
            </div>
          </div>

          <button
            className="btn btn-secondary"
            onClick={handleCheckBalance}
            disabled={isLoading}
            style={{ marginTop: 16 }}
          >
            {isLoading ? 'Loading...' : 'Refresh Balance'}
          </button>
        </div>
      )}

      <footer style={{ textAlign: 'center', marginTop: 32, color: 'var(--text-muted)' }}>
        <p>KaspaGraffiti v0.2.0 - Wallet Management & Address Derivation</p>
        <p style={{ fontSize: '0.875rem', marginTop: 8 }}>
          HD Wallet support enabled - BIP32/BIP44 compatible
        </p>
      </footer>
    </div>
  );
}

export default App;
