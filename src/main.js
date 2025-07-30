import {createApp} from "vue";
import App from "./App.vue";
import {router} from "./router";
import ArcoVue from '@arco-design/web-vue';
import '@arco-design/web-vue/dist/arco.css';
import "./style.css";
import {createPinia} from 'pinia'

// 主题管理
class ThemeManager {
    constructor() {
        this.currentTheme = localStorage.getItem('theme') || 'dark';
        this.init();
    }

    init() {
        this.applyTheme(this.currentTheme);
    }

    applyTheme(theme) {
        // 设置HTML根元素的data-theme属性
        document.documentElement.setAttribute('data-theme', theme);
        
        // 设置Arco Design的主题
        if (theme === 'dark') {
            document.body.setAttribute('arco-theme', 'dark');
        } else {
            document.body.removeAttribute('arco-theme');
        }
        
        // 保存到localStorage
        localStorage.setItem('theme', theme);
        this.currentTheme = theme;
    }

    toggleTheme() {
        const newTheme = this.currentTheme === 'dark' ? 'light' : 'dark';
        this.applyTheme(newTheme);
        return newTheme;
    }

    getTheme() {
        return this.currentTheme;
    }
}

// 创建全局主题管理器
const themeManager = new ThemeManager();

const pinia = createPinia()
const app = createApp(App)

// 将主题管理器添加到全局属性
app.config.globalProperties.$themeManager = themeManager;

app.use(router)
app.use(ArcoVue)
app.use(pinia)
app.mount("#app");
