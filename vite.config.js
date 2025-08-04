import {defineConfig} from "vite";
import vue from "@vitejs/plugin-vue";
import * as path from "path";
import VueSetupExtend from 'vite-plugin-vue-setup-extend'
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill'
import { NodeModulesPolyfillPlugin } from '@esbuild-plugins/node-modules-polyfill'
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [vue(), VueSetupExtend],
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
                manualChunks: (id) => {
                    // 第三方库分离
                    if (id.includes('node_modules')) {
                        // 加密相关库
                        if (id.includes('ethers')) {
                            return 'vendor-crypto';
                        }
                        // Vue生态系统
                        if (id.includes('vue') || id.includes('@vue')) {
                            return 'vendor-vue';
                        }
                        // Arco Design组件库
                        if (id.includes('@arco-design')) {
                            return 'vendor-arco';
                        }
                        // 图标库
                        if (id.includes('@iconify') || id.includes('iconify')) {
                            return 'vendor-icons';
                        }
                        // 工具库
                        if (id.includes('xlsx') || id.includes('pinia')) {
                            return 'vendor-utils';
                        }
                        // Tauri相关
                        if (id.includes('@tauri-apps')) {
                            return 'vendor-tauri';
                        }
                        // 其他第三方库
                        return 'vendor-misc';
                    }
                    
                    // 组件分离
                    if (id.includes('/components/')) {
                        // 管理类组件（较大的模态框组件）
                        if (id.includes('Management') || id.includes('Modal')) {
                            return 'components-management';
                        }
                        // 基础组件
                        return 'components-base';
                    }
                    
                    // 页面分离
                    if (id.includes('/pages/')) {
                        if (id.includes('Balance')) {
                            return 'page-balance';
                        }
                        if (id.includes('Transfer')) {
                            return 'page-transfer';
                        }
                        if (id.includes('Home')) {
                            return 'page-home';
                        }
                        return 'pages-misc';
                    }
                    
                    // 工具函数和存储
                    if (id.includes('/stores/') || id.includes('/utils/')) {
                        return 'app-core';
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
                drop_console: true,
                drop_debugger: true
            }
        },
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
        // 设置chunk大小警告限制
        chunkSizeWarningLimit: 1000,
        // 启用CSS代码分割
        cssCodeSplit: true
    }
});
