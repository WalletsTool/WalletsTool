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
        include: ['ethers', 'vue', 'vue-router'],
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
        port: 1420,
        strictPort: true,
    },
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
                // 重新启用手动分块，使用更安全的分块策略
                manualChunks: (id) => {
                    // 第三方库分块
                    if (id.includes('node_modules')) {
                        // Vue 相关库单独分块
                        if (id.includes('vue') || id.includes('@vue')) {
                            return 'vue-vendor';
                        }
                        // Arco Design 组件库单独分块
                        if (id.includes('@arco-design')) {
                            return 'arco-vendor';
                        }
                        // ethers和其他第三方库合并到vendor包中避免循环依赖
                        return 'vendor';
                    }
                    // 页面组件分块（兼容新结构）
                    if (
                        id.includes('/src/pages/') ||
                        id.includes('/src/views/') ||
                        id.includes('/src/features/')
                    ) {
                        return 'pages';
                    }
                    // 工具函数分块
                    if (id.includes('/src/utils/') || id.includes('/src/composables/')) {
                        return 'utils';
                    }
                    // 组件分块
                    if (id.includes('/src/components/')) {
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
        minify: !process.env.TAURI_DEBUG ? "terser" : false,
        terserOptions: {
            compress: {
                // 移除console和debugger语句
                drop_console: true,
                drop_debugger: true,
                // 死代码消除
                dead_code: true,
                // 移除未使用的变量和函数
                unused: true,
                // 函数内联优化
                inline: true,
                // 条件表达式优化
                conditionals: true,
                // 比较运算符优化
                comparisons: true,
                // 序列优化
                sequences: true,
                // 属性访问优化
                properties: true,
                // 循环优化
                loops: true,
                // 合并变量声明
                join_vars: true,
                // 移除无用代码
                side_effects: false,
                // 纯函数调用优化
                pure_funcs: ['console.log', 'console.info', 'console.debug', 'console.warn'],
                // 移除纯函数调用
                pure_getters: true,
                // 常量折叠
                evaluate: true,
                // 布尔值优化
                booleans: true,
                // 类型推断
                typeofs: true,
                // if语句优化
                if_return: true,
                // 表达式优化
                reduce_vars: true,
                // 移除重复代码
                collapse_vars: true
            }
        },
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
        // 设置chunk大小警告限制
        chunkSizeWarningLimit: 1000,
        // 小于4KB的资源将被内联为base64
        assetsInlineLimit: 4096,
        // 启用CSS代码分割
        cssCodeSplit: true
    }
});
