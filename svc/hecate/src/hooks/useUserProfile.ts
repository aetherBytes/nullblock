import { useState, useEffect } from 'react';
import { UserProfile } from '../types/user';
import { userApi } from '../common/services/user-api';

const CACHE_DURATION = 30 * 60 * 1000;
const CACHE_KEY = 'userProfile';
const CACHE_TIMESTAMP_KEY = 'userProfileTimestamp';

const getNetworkFromWalletType = (walletType: string | null): string => {
  switch (walletType) {
    case 'phantom':
      return 'solana';
    case 'metamask':
      return 'ethereum';
    default:
      return 'unknown';
  }
};

const inferNetworkFromAddress = (address: string): string => {
  if (/^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)) {
    return 'solana';
  }
  if (/^0x[a-fA-F0-9]{40}$/.test(address)) {
    return 'ethereum';
  }
  return 'unknown';
};

export const useUserProfile = (publicKey: string | null) => {
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchUserProfile = async () => {
    if (!publicKey) {
      console.log('👤 useUserProfile: No publicKey, skipping fetch');
      setUserProfile(null);
      return;
    }

    console.log('👤 useUserProfile: Starting fetch for', publicKey);

    const cached = localStorage.getItem(CACHE_KEY);
    const cacheTimestamp = localStorage.getItem(CACHE_TIMESTAMP_KEY);

    if (cached && cacheTimestamp) {
      const age = Date.now() - parseInt(cacheTimestamp, 10);
      if (age < CACHE_DURATION) {
        try {
          const parsed = JSON.parse(cached);
          if (parsed.source_identifier === publicKey) {
            setUserProfile(parsed);
            return;
          }
        } catch (err) {
          console.warn('Failed to parse cached user profile:', err);
        }
      }
    }

    setIsLoading(true);
    setError(null);

    try {
      const walletType = localStorage.getItem('walletType');
      let network = getNetworkFromWalletType(walletType);

      if (network === 'unknown') {
        console.warn('⚠️ walletType not found, attempting to infer from address');
        network = inferNetworkFromAddress(publicKey);

        if (network === 'unknown') {
          setError('Unable to determine wallet network. Please reconnect your wallet.');
          setIsLoading(false);
          return;
        }

        console.log('🔍 Inferred network from address:', network);
      }

      console.log('👤 useUserProfile: Calling API lookup with', { publicKey, network });
      const lookupResult = await userApi.lookupUser(publicKey, network);
      console.log('👤 useUserProfile: API response:', lookupResult);

      // Handle both response formats: {success, data} and {found, user}
      const isSuccess = lookupResult.success || (lookupResult as any).found;
      const userData = lookupResult.data || (lookupResult as any).user;

      if (isSuccess && userData) {
        console.log('✅ useUserProfile: Profile loaded successfully:', userData);
        setUserProfile(userData);
        localStorage.setItem(CACHE_KEY, JSON.stringify(userData));
        localStorage.setItem(CACHE_TIMESTAMP_KEY, Date.now().toString());
      } else {
        console.error('❌ useUserProfile: Profile lookup failed:', lookupResult.error);
        setError(lookupResult.error || 'User not found');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch user profile';
      setError(errorMessage);
      console.error('User profile fetch error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const refreshUserProfile = () => {
    localStorage.removeItem(CACHE_KEY);
    localStorage.removeItem(CACHE_TIMESTAMP_KEY);
    fetchUserProfile();
  };

  useEffect(() => {
    fetchUserProfile();

    const handleUserRegistered = (event: Event) => {
      const customEvent = event as CustomEvent;
      const { walletAddress } = customEvent.detail || {};

      if (walletAddress === publicKey) {
        console.log('🔄 User registered event received, refreshing profile...');

        localStorage.removeItem(CACHE_KEY);
        localStorage.removeItem(CACHE_TIMESTAMP_KEY);

        setTimeout(() => {
          fetchUserProfile();
        }, 500);
      }
    };

    window.addEventListener('user-registered', handleUserRegistered);

    return () => {
      window.removeEventListener('user-registered', handleUserRegistered);
    };
  }, [publicKey]);

  return {
    userProfile,
    isLoading,
    error,
    refreshUserProfile,
  };
};
