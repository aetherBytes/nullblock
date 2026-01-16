export const NULLBLOCK_OFFICIAL_WALLETS = [
  '0x7D05f6Be03D54cB2Ea05DD4b885A6f6da1Aafe8E',
  '6xu5aRG6z7ej3hKmQkv23cENWyxzMiFA49Ww1FRzmEaU',
  '0xcEcEe0C5f8d0d08F42727402b7081bf7Bc895D44',
  '5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT',
  '0x3be88EDa9E12ac15bBfD16Bb13eEFDd9871Bb6B7',
  'AncqdtRrVVbGjWCCv6z2gwL8SwrWTomyUbswJeCbe4vJ',
];

export interface NullBlockServiceCow {
  id: string;
  name: string;
  serviceDir: string;
  menuIcon: string;
  description: string;
}

export const NULLBLOCK_SERVICE_COWS: NullBlockServiceCow[] = [
  {
    id: 'arbfarm',
    name: 'ArbFarm',
    serviceDir: 'svc/arb-farm',
    menuIcon: '⚡',
    description: 'Solana MEV agent swarm',
  },
  {
    id: 'polymev',
    name: 'PolyMev',
    serviceDir: 'svc/poly-mev',
    menuIcon: '🎯',
    description: 'Polymarket trading agent',
  },
];

export function isNullBlockBranded(creatorWallet: string): boolean {
  return NULLBLOCK_OFFICIAL_WALLETS.includes(creatorWallet);
}

export function getNullBlockServiceCow(cowId: string): NullBlockServiceCow | undefined {
  return NULLBLOCK_SERVICE_COWS.find((cow) => cow.id === cowId);
}
