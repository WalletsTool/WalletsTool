import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import * as path from "path";
import VueSetupExtend from 'vite-plugin-vue-setup-extend'
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill'
import { NodeModulesPolyfillPlugin } from '@esbuild-plugins/node-modules-polyfill'
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [vue(), VueSetupExtend],
    // 设置base路径，Tauri应用需要相对路径
    base: process.env.TAURI_ENV_PLATFORM || process.env.TAURI_FAMILY ? './' : '/',
    // 设置路径别名
    resolve: {
        alias: [
            {
                find: '@',
                replacement: path.resolve('./src')
            }
        ]
    },
    optimizeDeps: {
        // 预构建依赖项，加速开发启动
        include: [
            'ethers',
            'vue',
            'vue-router',
            'pinia',
            '@arco-design/web-vue',
            'primevue',
            '@primevue/themes'
        ],
        // 依赖预构建后的缓存目录
        cacheDir: 'node_modules/.vite-cache',
        // esbuild 优化选项
        esbuildOptions: {
            // Node.js global to browser globalThis
            define: {
                global: 'globalThis'
            },
            // Enable esbuild polyfill plugins
            plugins: [
                NodeGlobalsPolyfillPlugin({
                    buffer: true,
                    process: true,
                }),
                NodeModulesPolyfillPlugin()
            ]
        }
    },
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1422,
        strictPort: false,
        // 允许访问项目根目录外的文件（用于Tauri）
        fs: {
            allow: ['.', '..'],
        },
        // 启用热更新优化
        hmr: {
            overlay: true,
        },
        // 开发服务器优化
        warmup: {
            // 预热常用的 transformed 文件
            transformedFiles: [
                '/src/main.js',
                '/src/App.vue',
                '/src/router/index.js'
            ]
        }
    },
    // 缓存配置
    cache: true,
    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ["VITE_", "TAURI_"],
    build: {
        rollupOptions: {
            plugins: [
                rollupNodePolyFill()
            ],
            output: {
                // 合并小chunk以减少HTTP请求数量
                experimentalMinChunkSize: 20000,
                // 手动分块策略 - 优化首屏加载
                manualChunks: (id) => {
                    // 第三方库分块 - 简化策略减少chunk数量
                    if (id.includes('node_modules')) {
                        if (id.includes('ethers')) {
                            return 'vendor-ethers';
                        }
                        if (id.includes('vue') || id.includes('vue-router') || id.includes('pinia')) {
                            return 'vendor-vue';
                        }
                        return 'vendor';
                    }
                    // 页面组件
                    if (
                        id.includes('/src/pages/') ||
                        id.includes('/src/views/') ||
                        id.includes('/src/features/')
                    ) {
                        return 'pages';
                    }
                    // 组件和工具
                    if (
                        id.includes('/src/components/') ||
                        id.includes('/src/utils/') ||
                        id.includes('/src/composables/')
                    ) {
                        return 'components';
                    }
                },
                // 优化chunk命名
                chunkFileNames: (chunkInfo) => {
                    const facadeModuleId = chunkInfo.facadeModuleId ? chunkInfo.facadeModuleId.split('/').pop().replace(/\.[^/.]+$/, '') : 'unknown';
                    return `js/[name]-[hash].js`;
                },
                entryFileNames: 'js/[name]-[hash].js',
                assetFileNames: (assetInfo) => {
                    const info = assetInfo.name.split('.');
                    const ext = info[info.length - 1];
                    if (/\.(css)$/.test(assetInfo.name)) {
                        return `css/[name]-[hash].${ext}`;
                    }
                    if (/\.(png|jpe?g|gif|svg|ico|webp)$/.test(assetInfo.name)) {
                        return `images/[name]-[hash].${ext}`;
                    }
                    if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name)) {
                        return `fonts/[name]-[hash].${ext}`;
                    }
                    return `assets/[name]-[hash].${ext}`;
                }
            }
        },
        // Tauri supports es2021
        target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari14",
        // 启用压缩优化
        minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
        // esbuild 压缩选项
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
        // 设置chunk大小警告限制（桌面应用无需过度拆分）
        chunkSizeWarningLimit: 2000,
        // 增大内联限制，减少HTTP请求
        assetsInlineLimit: 8192,
        // 启用CSS代码分割
        cssCodeSplit: true,
        // 禁用 treeshake 加速构建
        treeshake: false
    }
});
