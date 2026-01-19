import React from 'react';
import styles from '../arbfarm.module.scss';

export type ConfirmationType = 'danger' | 'warning';

export interface ConfirmationModalProps {
  isOpen: boolean;
  title: string;
  message: string;
  type?: ConfirmationType;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

const ICONS: Record<ConfirmationType, string> = {
  danger: '\u26A0',
  warning: '\u2139',
};

const ConfirmationModal: React.FC<ConfirmationModalProps> = ({
  isOpen,
  title,
  message,
  type = 'warning',
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  onConfirm,
  onCancel,
}) => {
  if (!isOpen) return null;

  const handleBackdropClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.target === e.currentTarget) {
      onCancel();
    }
  };

  return (
    <div className={styles.confirmationModal} onClick={handleBackdropClick}>
      <div className={styles.confirmationContent}>
        <div className={styles.confirmationHeader}>
          <div className={`${styles.confirmationIcon} ${styles[type]}`}>
            {ICONS[type]}
          </div>
          <h3>{title}</h3>
        </div>

        <p className={styles.confirmationMessage}>{message}</p>

        <div className={styles.confirmationActions}>
          <button className={styles.confirmCancelButton} onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            className={type === 'danger' ? styles.confirmDangerButton : styles.confirmWarningButton}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
};

export interface UseConfirmationOptions {
  title: string;
  message: string;
  type?: ConfirmationType;
  confirmLabel?: string;
  cancelLabel?: string;
}

export const useConfirmation = () => {
  const [config, setConfig] = React.useState<UseConfirmationOptions | null>(null);
  const [resolver, setResolver] = React.useState<((value: boolean) => void) | null>(null);

  const confirm = (options: UseConfirmationOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      setConfig(options);
      setResolver(() => resolve);
    });
  };

  const handleConfirm = () => {
    if (resolver) {
      resolver(true);
    }
    setConfig(null);
    setResolver(null);
  };

  const handleCancel = () => {
    if (resolver) {
      resolver(false);
    }
    setConfig(null);
    setResolver(null);
  };

  const ConfirmationDialog = config ? (
    <ConfirmationModal
      isOpen={true}
      title={config.title}
      message={config.message}
      type={config.type}
      confirmLabel={config.confirmLabel}
      cancelLabel={config.cancelLabel}
      onConfirm={handleConfirm}
      onCancel={handleCancel}
    />
  ) : null;

  return {
    confirm,
    ConfirmationDialog,
  };
};

export default ConfirmationModal;
