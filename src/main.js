import {createApp} from "vue";
import App from "./App.vue";
import {router} from "./router";
import ArcoVue from '@arco-design/web-vue';
import '@arco-design/web-vue/dist/arco.css';
import "./style.css";
import {createPinia} from 'pinia'

// 预加载关键资源
const preloadResources = () => {
  // 预加载字体
  const fontLink = document.createElement('link');
  fontLink.rel = 'preload';
  fontLink.as = 'font';
  fontLink.type = 'font/woff2';
  fontLink.crossOrigin = 'anonymous';
  document.head.appendChild(fontLink);
  
  // 预加载关键CSS
  const cssLink = document.createElement('link');
  cssLink.rel = 'preload';
  cssLink.as = 'style';
  cssLink.href = '/src/style.css';
  document.head.appendChild(cssLink);
};

// 优化字体加载
const optimizeFonts = () => {
  document.fonts.ready.then(() => {
    document.body.classList.add('fonts-loaded');
  });
};

const pinia = createPinia()
const app = createApp(App)

app.use(router)
app.use(ArcoVue)
app.use(pinia)

// 执行预加载和优化
preloadResources();
optimizeFonts();

app.mount("#app");
