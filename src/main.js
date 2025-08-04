import {createApp} from "vue";
import App from "./App.vue";
import {router} from "./router";
// 按需导入Arco Design组件
import {
  Button,
  Table,
  TableColumn,
  Modal,
  Form,
  FormItem,
  Input,
  InputNumber,
  Select,
  Option,
  Switch,
  Tag,
  Progress,
  Spin,
  Alert,
  Tooltip,
  Popconfirm,
  Divider,
  Row,
  Col,
  Radio,
  RadioGroup,
  Upload,
  Textarea,
  Dropdown,
  Doption,
  InputGroup,
  Notification,
  Message
} from '@arco-design/web-vue';
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
  
  // CSS已通过import语句正确导入，无需预加载
};

// 优化字体加载
const optimizeFonts = () => {
  document.fonts.ready.then(() => {
    document.body.classList.add('fonts-loaded');
  });
};

// 添加全局错误处理
window.addEventListener('error', (event) => {
  console.error('[DEBUG] 全局JavaScript错误:', {
    message: event.message,
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno,
    error: event.error
  })
})

window.addEventListener('unhandledrejection', (event) => {
  console.error('[DEBUG] 未处理的Promise拒绝:', event.reason)
})

console.log('[DEBUG] 开始创建Vue应用实例')
const pinia = createPinia()
const app = createApp(App)
console.log('[DEBUG] Vue应用实例创建成功')

// 添加Vue错误处理
app.config.errorHandler = (err, vm, info) => {
  console.error('[DEBUG] Vue错误处理器:', {
    error: err,
    component: vm,
    info: info
  })
}

// 注册Arco Design组件
console.log('[DEBUG] 开始注册Arco Design组件')
app.use(Button)
app.use(Table)
app.use(TableColumn)
app.use(Modal)
app.use(Form)
app.use(FormItem)
app.use(Input)
app.use(InputNumber)
app.use(Select)
app.use(Option)
app.use(Switch)
app.use(Tag)
app.use(Progress)
app.use(Spin)
app.use(Alert)
app.use(Tooltip)
app.use(Popconfirm)
app.use(Divider)
app.use(Row)
app.use(Col)
app.use(Radio)
app.use(RadioGroup)
app.use(Upload)
app.use(Textarea)
app.use(Dropdown)
app.use(Doption)
app.use(InputGroup)

console.log('[DEBUG] 开始注册路由和状态管理')
app.use(router)
app.use(pinia)
console.log('[DEBUG] 路由和状态管理注册完成')

// 执行预加载和优化
console.log('[DEBUG] 开始预加载资源')
preloadResources();
optimizeFonts();
console.log('[DEBUG] 资源预加载完成')

console.log('[DEBUG] 开始挂载Vue应用到#app')
app.mount("#app");
console.log('[DEBUG] Vue应用挂载完成')
