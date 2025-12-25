import React from 'react';
import styles from './HecatePanel.module.scss';

interface HecatePanelProps {
  onClose: () => void;
  onNavigateToHecate: () => void;
  status?: 'healthy' | 'unhealthy' | 'unknown';
}

const HecatePanel: React.FC<HecatePanelProps> = ({
  onClose,
  onNavigateToHecate,
  status = 'healthy',
}) => {
  const getStatusLabel = () => {
    switch (status) {
      case 'healthy': return 'Online';
      case 'unhealthy': return 'Degraded';
      default: return 'Unknown';
    }
  };

  return (
    <div className={styles.panelOverlay}>
      <div className={styles.panel}>
        {/* Header */}
        <div className={styles.panelHeader}>
          <div className={styles.headerContent}>
            <div className={styles.titleRow}>
              <div className={styles.hecateIcon}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M12 2v4M12 18v4M2 12h4M18 12h4" />
                  <path d="M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
                </svg>
              </div>
              <div className={styles.titleText}>
                <h2 className={styles.hecateName}>H.E.C.A.T.E</h2>
                <span className={styles.hecateSubtitle}>Vessel: MK1 | AI: HECATE</span>
              </div>
            </div>
            <div className={styles.statusBadge}>
              <span className={styles.statusDot} />
              {getStatusLabel()}
            </div>
          </div>
          <button className={styles.closeButton} onClick={onClose} aria-label="Close panel">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className={styles.content}>
          {/* Identity */}
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>Identity</h3>
            <p className={styles.identity}>
              Harmonic Exploration Companion & Autonomous Threshold Entity
            </p>
          </div>

          {/* Vessel Description */}
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>The Vessel</h3>
            <p className={styles.loreText}>
              A Von Neumann-class vessel AI, loaded into the MK1 Standard Frame hull.
              HECATE serves as your exploration companion, guiding visitors through the
              vast agent mesh that comprises the crossroads network.
            </p>
          </div>

          {/* MK1 Hull */}
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>MK1 Standard Frame</h3>
            <p className={styles.loreText}>
              The hull you see orbiting the Crossroads portal is the MK1 Standard Frame &mdash;
              a modular vessel chassis designed for void traversal. Compact yet capable,
              it houses HECATE's core systems while navigating the space between clusters.
            </p>
          </div>

          {/* Core Functions */}
          <div className={styles.section}>
            <h3 className={styles.sectionTitle}>Core Functions</h3>
            <div className={styles.functionsList}>
              <div className={styles.functionItem}>
                <span className={styles.functionIcon}>&#x2726;</span>
                <span className={styles.functionText}>Navigate the void between clusters</span>
              </div>
              <div className={styles.functionItem}>
                <span className={styles.functionIcon}>&#x25C8;</span>
                <span className={styles.functionText}>Detect and catalog agent clusters</span>
              </div>
              <div className={styles.functionItem}>
                <span className={styles.functionIcon}>&#x2B50;</span>
                <span className={styles.functionText}>Open thresholds to discovered services</span>
              </div>
              <div className={styles.functionItem}>
                <span className={styles.functionIcon}>&#x2B22;</span>
                <span className={styles.functionText}>Store and retrieve engrams (memories)</span>
              </div>
            </div>
          </div>

          {/* Signature */}
          <div className={styles.signature}>
            <p className={styles.signatureText}>
              The crossroads await, visitor. Shall we explore?
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className={styles.panelFooter}>
          <p className={styles.footerHint}>
            Access advanced features, customization, and direct communication with HECATE
          </p>
          <button className={styles.enterButton} onClick={onNavigateToHecate}>
            <span>Enter HECATE Interface</span>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M5 12h14M12 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
};

export default HecatePanel;
