import { invoke } from '@tauri-apps/api/core';

export interface RecordingSession {
  id: string;
  url: string;
  startTime: Date;
  actions: RecordedAction[];
  status: 'recording' | 'stopped' | 'error';
}

export interface RecordedAction {
  type: 'click' | 'fill' | 'navigate' | 'select' | 'hover' | 'screenshot' | 'upload' | 'evaluate';
  selector?: string;
  value?: string;
  timestamp: Date;
  description: string;
}

export interface RecordingOptions {
  browserType?: 'chromium' | 'firefox' | 'webkit';
  headless?: boolean;
  viewportWidth?: number;
  viewportHeight?: number;
  includeComments?: boolean;
  extensions?: string[];
}

class PlaywrightRecorderBridge {
  private currentSession: RecordingSession | null = null;
  private sessionId: string | null = null;
  private currentExtensions: string[] = [];

  async startSession(url: string, options: RecordingOptions = {}): Promise<RecordingSession> {
    const {
      browserType = 'chromium',
      headless = false,
      viewportWidth = 1280,
      viewportHeight = 720,
      includeComments = true,
      extensions = []
    } = options;

    this.currentExtensions = extensions;
    this.currentSession = {
      id: `session-${Date.now()}`,
      url,
      startTime: new Date(),
      actions: [],
      status: 'recording'
    };

    try {
      if (typeof window !== 'undefined' && (window as any).__MCP_PLAYWRIGHT__) {
        const mcp = (window as any).__MCP_PLAYWRIGHT__;
        
        await mcp.playwright_navigate({
          url,
          browserType,
          headless,
          width: viewportWidth,
          height: viewportHeight
        });

        const result = await mcp.start_codegen_session({
          options: {
            outputPath: '',
            testNamePrefix: 'RecordedScript',
            includeComments
          }
        });

        if (result && result.sessionId) {
          this.sessionId = result.sessionId;
        }

        this.addAction('navigate', `导航到 ${url}`, { url });
        
        if (extensions.length > 0) {
          this.addAction('evaluate', `已加载 ${extensions.length} 个浏览器插件`, {});
        }
      }

      return this.currentSession;
    } catch (error) {
      this.currentSession.status = 'error';
      throw error;
    }
  }

  async stopSession(): Promise<string | null> {
    if (!this.currentSession || !this.sessionId) {
      return null;
    }

    try {
      let generatedCode = '';

      if (typeof window !== 'undefined' && (window as any).__MCP_PLAYWRIGHT__) {
        const mcp = (window as any).__MCP_PLAYWRIGHT__;
        
        const result = await mcp.end_codegen_session({
          sessionId: this.sessionId
        });

        if (result && result.code) {
          generatedCode = this.convertToScriptFormat(result.code);
        }

        await mcp.playwright_close();
      }

      this.currentSession.status = 'stopped';
      this.sessionId = null;

      return generatedCode;
    } catch (error) {
      this.currentSession.status = 'error';
      throw error;
    }
  }

  addAction(type: RecordedAction['type'], description: string, details: { selector?: string; value?: string } = {}): void {
    if (!this.currentSession) return;

    this.currentSession.actions.push({
      type,
      selector: details.selector,
      value: details.value,
      timestamp: new Date(),
      description
    });
  }

  getCurrentSession(): RecordingSession | null {
    return this.currentSession;
  }

  clearSession(): void {
    this.currentSession = null;
    this.sessionId = null;
    this.currentExtensions = [];
  }

  getExtensions(): string[] {
    return this.currentExtensions;
  }

  private convertToScriptFormat(playwrightCode: string): string {
    let code = playwrightCode;
    
    code = code.replace(/const { chromium } = require\('playwright'\);?\n?/g, '');
    code = code.replace(/\(async \(\) => \{[\s\S]*?try \{[\s\S]*?const browser = await chromium\.launch\([^)]*\);?\n?/g, '');
    code = code.replace(/const context = await browser\.newContext\([^)]*\);?\n?/g, '');
    code = code.replace(/const page = await context\.newPage\([^)]*\);?\n?/g, '');
    code = code.replace(/await browser\.close\(\);?\n?/g, '');
    code = code.replace(/\} catch[^}]*\}[\s\S]*?\}\)\(\);?/g, '');
    code = code.replace(/^\s*\n/gm, '');

    const lines = code.split('\n').filter(line => line.trim());
    const indentedLines = lines.map(line => '    ' + line);

    const actionCount = this.currentSession?.actions.length || 0;
    const extensionInfo = this.currentExtensions.length > 0 
      ? `// 加载插件: ${this.currentExtensions.length} 个` 
      : '';

    return `// 录制生成的脚本
// 录制时间: ${new Date().toLocaleString()}
// 操作数量: ${actionCount}
${extensionInfo}

async function run({ page, wallet, api }) {
    api.log('info', '开始执行录制脚本');
    
${indentedLines.join('\n')}
    
    api.log('success', '脚本执行完成');
    return { success: true };
}`;
  }
}

export const playwrightRecorder = new PlaywrightRecorderBridge();

export const recorderService = {
  async startRecording(url: string, options?: RecordingOptions): Promise<RecordingSession> {
    return await playwrightRecorder.startSession(url, options);
  },

  async stopRecording(): Promise<string | null> {
    return await playwrightRecorder.stopSession();
  },

  getCurrentSession(): RecordingSession | null {
    return playwrightRecorder.getCurrentSession();
  },

  clearRecording(): void {
    playwrightRecorder.clearSession();
  },

  addAction(type: RecordedAction['type'], description: string, details?: { selector?: string; value?: string }): void {
    playwrightRecorder.addAction(type, description, details);
  }
};
