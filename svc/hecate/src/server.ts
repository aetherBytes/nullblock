import { StreamSuspense } from '@lomray/consistent-suspense/server';
import { Manager as MetaManager } from '@lomray/react-head-manager';
import MetaServer from '@lomray/react-head-manager/server';
import { Manager } from '@lomray/react-mobx-manager';
import ManagerStream from '@lomray/react-mobx-manager/manager-stream';
import entryServer from '@lomray/vite-ssr-boost/node/entry';
import CookieParser from 'cookie-parser';
import { createIsbotFromList, list, isbotPatterns } from 'isbot';
import { enableStaticRendering } from 'mobx-react-lite';
import StateKey from '@constants/state-key';
import routes from '@routes/index';
import App from './app';

// Enhanced logging utility
const log = {
  info: (message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`🌐 [${timestamp}] ℹ️  ${message}`, data ? data : '');
  },
  success: (message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`🌐 [${timestamp}] ✅ ${message}`, data ? data : '');
  },
  warning: (message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`🌐 [${timestamp}] ⚠️  ${message}`, data ? data : '');
  },
  error: (message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    console.log(`🌐 [${timestamp}] ❌ ${message}`, data ? data : '');
  },
  request: (method: string, url: string, userAgent?: string) => {
    const timestamp = new Date().toISOString();
    console.log(`🌐 [${timestamp}] 📥 ${method} ${url} ${userAgent ? `(${userAgent.substring(0, 50)}...)` : ''}`);
  },
  response: (statusCode: number, url: string, duration?: number) => {
    const timestamp = new Date().toISOString();
    const emoji = statusCode >= 200 && statusCode < 300 ? '✅' : statusCode >= 400 ? '❌' : '⚠️';
    console.log(`🌐 [${timestamp}] ${emoji} ${statusCode} ${url} ${duration ? `(${duration}ms)` : ''}`);
  }
};

/**
 * Exclude AWS Amplify user agent
 */
const patternsToRemove = new Set(['Amazon CloudFront'].map(isbotPatterns).flat());
const isBot = createIsbotFromList(list.filter((record) => !patternsToRemove.has(record)));

// noinspection JSUnusedGlobalSymbols
/**
 * Configure server
 */
export default entryServer(App, routes, {
  abortDelay: 20000,
  init: () => ({
    /**
     * Once after create server
     */
    onServerCreated: (app) => {
      log.success('🚀 Hecate Server Created Successfully');
      log.info('📋 Server Configuration:', {
        abortDelay: 20000,
        patternsToRemove: Array.from(patternsToRemove),
        isBotPatterns: list.length
      });

      enableStaticRendering(true);
      app.use(CookieParser());

      // Add request logging middleware
      app.use((req, res, next) => {
        const start = Date.now();
        const userAgent = req.get('user-agent') || 'Unknown';
        
        log.request(req.method, req.url, userAgent);
        
        res.on('finish', () => {
          const duration = Date.now() - start;
          log.response(res.statusCode, req.url, duration);
        });
        
        next();
      });

      log.success('🔧 Middleware and Static Rendering Configured');
    },
    /**
     * For each request:
     * 1. Create mobx manager
     * 2. Create meta manager
     * 3. Listen stream to add mobx suspense stores to output
     */
    onRequest: async () => {
      log.info('🔄 Processing New Request - Initializing Managers');
      
      try {
        const storeManager = new Manager({
          options: { shouldDisablePersist: true },
        });
        const storeManageStream = new ManagerStream(storeManager);
        const metaManager = new MetaManager();

        log.info('📦 Store Manager Created');
        await storeManager.init();
        log.success('✅ Store Manager Initialized');

        const streamSuspense = StreamSuspense.create((suspenseId) =>
          storeManageStream.take(suspenseId),
        );
        log.success('🌊 Stream Suspense Created');

        log.success('🎯 Request Processing Complete - All Managers Ready');
        
        return {
          appProps: {
            storeManager,
            metaManager,
            streamSuspense,
          },
          // disable 103 early hints for AWS ALB
          hasEarlyHints: false,
        };
      } catch (error) {
        log.error('❌ Failed to Initialize Request Managers', error);
        throw error;
      }
    },
    /**
     * We can control stream mode here
     */
    onRouterReady: ({ context: { req } }) => {
      const userAgent = req.get('user-agent') || '';
      const isStream = !isBot(userAgent) && req.cookies?.isCrawler !== '1';
      
      log.info('🎯 Router Ready - Determining Stream Mode', {
        userAgent: userAgent.substring(0, 100),
        isBot: isBot(userAgent),
        isCrawler: req.cookies?.isCrawler,
        isStream
      });

      return {
        isStream,
      };
    },
    /**
     * Inject header meta tags
     */
    onShellReady: ({
      context: {
        appProps: { metaManager },
        html: { header },
      },
    }) => {
      log.info('📄 Shell Ready - Injecting Meta Tags');
      
      try {
        const newHead = MetaServer.inject(header, metaManager);
        log.success('✅ Meta Tags Injected Successfully');
        
        return {
          header: newHead,
        };
      } catch (error) {
        log.error('❌ Failed to Inject Meta Tags', error);
        throw error;
      }
    },
    /**
     * Analyze react stream output and return additional html from callback `onRequest` in StreamSuspense
     */
    onResponse: ({
      context: {
        appProps: { streamSuspense },
        isStream,
      },
      html,
    }) => {
      if (!isStream) {
        log.info('📤 Non-Stream Response - Skipping Stream Analysis');
        return;
      }

      log.info('🌊 Analyzing Stream Response');
      try {
        const result = streamSuspense.analyze(html);
        log.success('✅ Stream Analysis Complete');
        return result;
      } catch (error) {
        log.error('❌ Stream Analysis Failed', error);
        throw error;
      }
    },
    /**
     * Return server state to client (once when app she'll ready) for:
     * 1. Mobx manager (stores)
     * 2. Meta manager
     */
    getState: ({
      context: {
        appProps: { storeManager, metaManager },
      },
    }) => {
      log.info('📦 Preparing Server State for Client');
      
      try {
        const storeState = storeManager.toJSON();
        const metaState = MetaServer.getState(metaManager);
        
        log.success('✅ Server State Prepared', {
          storeStateKeys: Object.keys(storeState),
          metaStateKeys: Object.keys(metaState)
        });

        return {
          [StateKey.storeManager]: storeState,
          [StateKey.metaManager]: metaState,
        };
      } catch (error) {
        log.error('❌ Failed to Prepare Server State', error);
        throw error;
      }
    },
  }),
});
