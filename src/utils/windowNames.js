export const WINDOW_CONFIG = {
  app: {
    name: 'WalletsTool'
  },
  
  separator: {
    main: ' | ',
    instance: ' [',
    instanceEnd: ']'
  },

  icons: {
    tray: '🔗',
    transfer: '💸',
    balance: '💰',
    monitor: '👁️'
  },

  modules: {
    transfer: {
      name: '批量转账',
      shortName: '转账',
      icon: 'transfer',
      businessLabels: ['A组', 'B组', 'C组', '冷钱包', '热钱包', '交易所', '个人', '测试']
    },
    balance: {
      name: '余额查询',
      shortName: '查询',
      icon: 'balance',
      businessLabels: ['大额地址', '空投检查', '代币分布', '资产统计', 'VIP检查']
    },
    monitor: {
      name: '链上监控',
      shortName: '监控',
      icon: 'monitor',
      businessLabels: ['大额监控', '新地址', '活动监控', '异常监控']
    }
  },

  generateTitle(moduleKey, businessLabel = null, instanceNum = 1) {
    const module = this.modules[moduleKey]
    if (!module) return `${this.app.name} - 未知功能`

    const icon = this.icons[module.icon] || ''
    let title = `${this.app.name} - ${icon} ${module.name}`

    if (businessLabel) {
      title += `${this.separator.main}${businessLabel}`
    }

    if (instanceNum > 1) {
      title += `${this.separator.instance}${instanceNum}${this.separator.instanceEnd}`
    }

    return title
  },

  generateTrayTitle(moduleKey, instanceNum = 1) {
    const module = this.modules[moduleKey]
    if (!module) return `${this.icons.tray} ${this.app.name} - 未知功能`

    const icon = this.icons[module.icon] || ''
    return `${this.icons.tray} ${icon} ${module.shortName} [${instanceNum}]`
  },

  generateLabel(pageName, instanceNum) {
    return `${pageName}${instanceNum}`
  },

  suggestBusinessLabel(moduleKey, existingLabels = []) {
    const module = this.modules[moduleKey]
    if (!module) return null

    const available = module.businessLabels.filter(label => !existingLabels.includes(label))
    if (available.length > 0) {
      return available[0]
    }

    return `窗口${existingLabels.length + 1}`
  },

  STORAGE_KEY: 'wallets_tool_window_titles',

  getAllCustomTitles() {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY)
      return stored ? JSON.parse(stored) : {}
    } catch (e) {
      console.error('获取自定义标题失败:', e)
      return {}
    }
  },

  saveCustomTitle(windowLabel, customTitle) {
    if (!windowLabel || !customTitle) return false
    
    try {
      const titles = this.getAllCustomTitles()
      titles[windowLabel] = customTitle.trim()
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(titles))
      return true
    } catch (e) {
      console.error('保存自定义标题失败:', e)
      return false
    }
  },

  getCustomTitle(windowLabel) {
    if (!windowLabel) return null
    
    try {
      const titles = this.getAllCustomTitles()
      return titles[windowLabel] || null
    } catch (e) {
      console.error('获取自定义标题失败:', e)
      return null
    }
  },

  removeCustomTitle(windowLabel) {
    if (!windowLabel) return false
    
    try {
      const titles = this.getAllCustomTitles()
      if (titles[windowLabel]) {
        delete titles[windowLabel]
        localStorage.setItem(this.STORAGE_KEY, JSON.stringify(titles))
      }
      return true
    } catch (e) {
      console.error('删除自定义标题失败:', e)
      return false
    }
  },

  getDisplayTitle(moduleKey, windowLabel, instanceNum = 1) {
    const customTitle = this.getCustomTitle(windowLabel)
    if (customTitle) {
      return customTitle
    }
    
    const module = this.modules[moduleKey]
    if (!module) return `${this.app.name} - 未知功能`
    
    const icon = this.icons[module.icon] || ''
    let title = `${this.app.name} - ${icon} ${module.name}`
    
    if (instanceNum > 1) {
      title += `${this.separator.instance}${instanceNum}${this.separator.instanceEnd}`
    }
    
    return title
  },

  isCustomTitle(windowLabel) {
    return this.getCustomTitle(windowLabel) !== null
  },

  clearAllCustomTitles() {
    try {
      localStorage.removeItem(this.STORAGE_KEY)
      return true
    } catch (e) {
      console.error('清除所有自定义标题失败:', e)
      return false
    }
  },

  getTitleStats() {
    try {
      const titles = this.getAllCustomTitles()
      const keys = Object.keys(titles)
      return {
        total: keys.length,
        labels: keys
      }
    } catch (e) {
      return { total: 0, labels: [] }
    }
  }
}
