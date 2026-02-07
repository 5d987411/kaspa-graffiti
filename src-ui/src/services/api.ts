export interface WalletInfo {
  private_key: string;
  public_key: string;
  address: string;
  network: string;
}

export interface BalanceInfo {
  balance: number;
  address: string;
}

export interface UtxoInfo {
  txid: string;
  vout: number;
  amount: number;
  script_pubkey: string;
}

export interface SendResult {
  txid: string;
  fee: number;
  change: number;
  address: string;
}

export interface HDWalletInfo {
  seed: string;
  address: string;
  network: string;
}

export interface DerivedAddressInfo {
  address: string;
  index: number;
  is_change: boolean;
  private_key: string;
  public_key: string;
}

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
const invoke = isTauri ? (window as any).__TAURI__.core.invoke : null;

let mockAddress = '';

const CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
// Use local CORS proxy in browser mode, direct API in Tauri
const PUBLIC_API = (typeof window !== 'undefined' && !(window as any).__TAURI__) 
  ? `${window.location.origin}/api` 
  : 'https://api-tn10.kaspa.org';

function generateRandomHex(length: number): string {
  const chars = '0123456789abcdef';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)];
  }
  return result;
}

function generateRandomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

function bech32Polymod(values: Uint8Array): bigint {
  let c = 1n;
  for (let i = 0; i < values.length; i++) {
    const c0 = c >> 35n;  // FIXED: Use c0 not top
    c = ((c & 0x07ffffffffn) << 5n) ^ BigInt(values[i]);  // FIXED: 35-bit mask
    if (c0 & 1n) c ^= 0x98f2bc8e61n;
    if (c0 & 2n) c ^= 0x79b76d99e2n;
    if (c0 & 4n) c ^= 0xf33e5fb3c4n;
    if (c0 & 8n) c ^= 0xae2eabe2a8n;
    if (c0 & 16n) c ^= 0x1e4f43e470n;
  }
  return c ^ 1n;
}

function conv8to5(data: Uint8Array): Uint8Array {
  let result: number[] = [];
  let buff = 0;
  let bits = 0;
  
  for (let i = 0; i < data.length; i++) {
    buff = (buff << 8) | data[i];
    bits += 8;
    while (bits >= 5) {
      bits -= 5;
      result.push((buff >> bits) & 0x1F);
      buff &= (1 << bits) - 1;
    }
  }
  if (bits > 0) {
    result.push((buff << (5 - bits)) & 0x1F);
  }
  
  return new Uint8Array(result);
}

function bech32Encode(hrp: string, data: Uint8Array): string {
  const hrp5 = hrp.split('').map(c => c.charCodeAt(0) & 0x1F);
  const data5 = conv8to5(data);
  
  // Calculate checksum: polymod(hrp5 + [0] + data5 + [0,0,0,0,0,0,0,0])
  const checksumInput = [...hrp5, 0, ...data5, 0, 0, 0, 0, 0, 0, 0, 0];
  const checksum = bech32Polymod(new Uint8Array(checksumInput));
  
  // Convert checksum to 5-bit (last 5 bytes of 8-byte big-endian)
  const checksumBytes = new Uint8Array(8);
  for (let i = 0; i < 8; i++) {
    checksumBytes[7 - i] = Number((checksum >> BigInt(i * 8)) & 0xFFn);
  }
  const checksum5 = conv8to5(checksumBytes.slice(3));
  
  let result = hrp + ':';
  for (let i = 0; i < data5.length; i++) {
    result += CHARSET[data5[i]];
  }
  for (let i = 0; i < checksum5.length; i++) {
    result += CHARSET[checksum5[i]];
  }
  
  return result;
}

async function kaspaAddressFromPublicKey(publicKeyBytes: Uint8Array, network: string = 'testnet-10'): Promise<string> {
  // Official Kaspa format: Use x-only public key directly (32 bytes, strip 02/03 prefix)
  // Payload structure: [version: 1 byte][xonly_pubkey: 32 bytes] = 33 bytes total
  
  // Extract x-only pubkey (remove first byte which is 02 or 03)
  const xonlyPubkey = publicKeyBytes.slice(1);
  
  if (xonlyPubkey.length !== 32) {
    throw new Error(`Invalid x-only pubkey length: ${xonlyPubkey.length}, expected 32`);
  }

  // Create payload: version byte (0 = PubKey) + x-only pubkey
  const payload = new Uint8Array(33);
  payload[0] = 0; // Version 0 = PubKey
  payload.set(xonlyPubkey, 1);

  const hrp = network === 'mainnet' ? 'kaspa' :
              network.startsWith('testnet') ? 'kaspatest' : 'kaspasim';

  return bech32Encode(hrp, payload);
}

async function generateKaspaAddress(): Promise<{ privateKey: string; publicKey: string; address: string }> {
  const privateKeyBytes = generateRandomBytes(32);
  const privateKey = Array.from(privateKeyBytes).map(b => b.toString(16).padStart(2, '0')).join('');

  // Generate compressed public key (33 bytes: 0x02/0x03 + 32 bytes X coordinate)
  // For demo, we use random bytes - in production this should derive from private key using secp256k1
  const publicKeyBytes = generateRandomBytes(33);
  publicKeyBytes[0] = 0x02; // Even Y coordinate prefix

  const publicKey = Array.from(publicKeyBytes).map(b => b.toString(16).padStart(2, '0')).join('');
  const address = await kaspaAddressFromPublicKey(publicKeyBytes);

  return { privateKey, publicKey, address };
}

async function fetchBalanceFromApi(address: string): Promise<BalanceInfo> {
  try {
    const encodedAddress = encodeURIComponent(address);
    const response = await fetch(`${PUBLIC_API}/addresses/${encodedAddress}/balance`);
    
    if (response.ok) {
      const data = await response.json();
      return { balance: Number(data.balance || 0), address };
    }
    
    const utxosResponse = await fetch(`${PUBLIC_API}/addresses/${encodedAddress}/utxos`);
    if (utxosResponse.ok) {
      const data = await utxosResponse.json();
      const entries = data.entries || [];
      const totalBalance = entries.reduce((sum: number, entry: any) => sum + Number(entry.utxo_entry?.amount || 0), 0);
      return { balance: totalBalance, address };
    }
    
    return { balance: 0, address };
  } catch (e) {
    console.warn('API fetch failed:', e);
    return { balance: 0, address };
  }
}

async function fetchUtxosFromApi(address: string): Promise<UtxoInfo[]> {
  try {
    const encodedAddress = encodeURIComponent(address);
    const response = await fetch(`${PUBLIC_API}/addresses/${encodedAddress}/utxos`);
    
    if (response.ok) {
      const data = await response.json();
      const entries = Array.isArray(data) ? data : (data.entries || []);
      
      return entries.map((entry: any) => ({
        txid: entry.outpoint?.transactionId || entry.outpoint?.transaction_id || '',
        vout: entry.outpoint?.index || 0,
        amount: Number(entry.utxoEntry?.amount || entry.utxo_entry?.amount || 0),
        script_pubkey: entry.utxoEntry?.scriptPublicKey?.scriptPublicKey || 
                      entry.utxo_entry?.script_public_key?.script || '',
      }));
    }
    
    return [];
  } catch (e) {
    console.warn('API fetch failed:', e);
    return [];
  }
}

export const walletGenerate = async (): Promise<WalletInfo> => {
  if (!isTauri || !invoke) {
    const { privateKey, publicKey, address } = await generateKaspaAddress();
    mockAddress = address;
    return {
      private_key: privateKey,
      public_key: publicKey,
      address,
      network: 'testnet-10',
    };
  }
  return await invoke('wallet_generate');
};

export const walletLoad = async (privateKey: string): Promise<WalletInfo> => {
  if (!isTauri || !invoke) {
    const cleanKey = privateKey.replace(/^(kaspa:|kaspatest:|Kaspa:|Kaspatest:)/g, '').trim();
    const { publicKey, address } = await generateKaspaAddress();
    mockAddress = address;
    return {
      private_key: cleanKey,
      public_key: publicKey,
      address,
      network: 'testnet-10',
    };
  }
  return await invoke('wallet_load', { privateKey });
};

export const addressValidate = async (address: string): Promise<boolean> => {
  if (!isTauri || !invoke) {
    return (address.startsWith('kaspa:') || address.startsWith('kaspatest:')) && address.length >= 60;
  }
  return await invoke('address_validate', { address });
};

export const balanceGet = async (address: string, rpcUrl?: string): Promise<BalanceInfo> => {
  if (!isTauri || !invoke) {
    return await fetchBalanceFromApi(address);
  }
  return await invoke('balance_get', { address, rpcUrl });
};

export const utxosGet = async (address: string, rpcUrl?: string): Promise<UtxoInfo[]> => {
  if (!isTauri || !invoke) {
    return await fetchUtxosFromApi(address);
  }
  return await invoke('utxos_get', { address, rpcUrl });
};

export const graffitiSend = async (
  privateKey: string,
  message: string,
  mimetype: string | null,
  rpcUrl: string | null,
  feeRate: number
): Promise<SendResult> => {
  if (!isTauri || !invoke) {
    const txid = 'mock_' + generateRandomHex(16);
    return {
      txid,
      fee: 1000,
      change: 99000,
      address: mockAddress,
    };
  }
  return await invoke('graffiti_send', {
    privateKey,
    message,
    mimetype,
    rpcUrl,
    feeRate,
  });
};

export const greet = async (name: string): Promise<string> => {
  if (!isTauri || !invoke) {
    return `Hello, ${name}! (browser mode - REST API for testnet-10)`;
  }
  return await invoke('greet', { name });
};

export const walletHdGenerate = async (): Promise<HDWalletInfo> => {
  if (!isTauri || !invoke) {
    const seedBytes = crypto.getRandomValues(new Uint8Array(32));
    const seed = Array.from(seedBytes).map(b => b.toString(16).padStart(2, '0')).join('');
    return {
      seed,
      address: 'kaspatest:pending',
      network: 'testnet-10',
    };
  }
  return await invoke('wallet_hd_generate');
};

export const walletHdLoad = async (seed: string): Promise<HDWalletInfo> => {
  if (!isTauri || !invoke) {
    return {
      seed,
      address: 'kaspatest:pending',
      network: 'testnet-10',
    };
  }
  return await invoke('wallet_hd_load', { seed });
};

export const deriveAddress = async (seed: string, index: number, isChange?: boolean): Promise<DerivedAddressInfo> => {
  if (!isTauri || !invoke) {
    return {
      address: 'kaspatest:pending',
      index,
      is_change: isChange || false,
      private_key: '0000000000000000000000000000000000000000000000000000000000000000',
      public_key: '0000000000000000000000000000000000000000000000000000000000000000',
    };
  }
  return await invoke('derive_address', { seed, index, change: isChange });
};

export const deriveMany = async (privateKey: string, count: number): Promise<DerivedAddressInfo[]> => {
  if (!isTauri || !invoke) {
    return [];
  }
  return await invoke('derive_many', { privateKey, count });
};
