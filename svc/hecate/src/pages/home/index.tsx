import { Meta } from '@lomray/react-head-manager';
import { IS_SSR_MODE } from '@lomray/vite-ssr-boost/constants/common';
import type { FCRoute } from '@lomray/vite-ssr-boost/interfaces/fc-route';
import cn from 'classnames';
import Cookies from 'js-cookie';
import { useState } from 'react';
import { Link, useLoaderData } from 'react-router-dom';
import ReactLogoImg from '@assets/images/react.svg';
import { APP_VERSION, IS_PROD } from '@constants/index';
import RouteManager from '@services/route-manager';
import styles from './styles.module.scss';

interface ILoaderData {
  isDefaultCrawler: boolean;
}

/**🧙
 * Home page
 * @constructor
 */
const Home: FCRoute = () => {
  const { isDefaultCrawler } = useLoaderData() as ILoaderData;
  const [isCrawler, setIsCrawler] = useState(isDefaultCrawler);

  // show only on deployed application
  const hasVersion = IS_PROD && !APP_VERSION.startsWith('APP_');

  /**
   * Enable/disable stream for demo
   */
  const toggleCrawler = () => {
    const nextVal = isCrawler ? '0' : '1';

    setIsCrawler(nextVal === '1');
    Cookies.set('isCrawler', nextVal);
  };

  return (
    <div>
      {/* Optional: Content or actions based on isCrawler state */}
      {/* The background image styling remains in your CSS/SCSS */}
    </div>
  );
};

Home.loader = ({ request }): ILoaderData => {
  const isDefaultCrawler =
    request.headers.get('cookie')?.includes('isCrawler=1') ?? Cookies.get('isCrawler') === '1';

  return {
    isDefaultCrawler,
  };
};

export default Home;
