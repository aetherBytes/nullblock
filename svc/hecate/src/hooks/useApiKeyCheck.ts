import { useState, useEffect } from 'react';

const EREBUS_API_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

export const useApiKeyCheck = (userId: string | null) => {
  const [hasApiKeys, setHasApiKeys] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!userId) {
      setHasApiKeys(false);
      setIsLoading(false);

      return;
    }

    fetch(`${EREBUS_API_URL}/api/users/${userId}/api-keys`)
      .then((res) => res.json())
      .then((data) => {
        const hasKeys = data.success && data.data.length > 0;

        setHasApiKeys(hasKeys);
        setIsLoading(false);
      })
      .catch(() => {
        setHasApiKeys(false);
        setIsLoading(false);
      });
  }, [userId]);

  return { hasApiKeys, isLoading };
};
