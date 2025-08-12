import { defineStore } from 'pinia'

export const useEcosystemStore = defineStore('ecosystem', {
  state: () => ({
    currentEco: localStorage.getItem('ecosystem') || 'eth',
    supported: ['eth', 'sol'],
  }),
  actions: {
    setEco(eco) {
      if (!this.supported.includes(eco)) return
      this.currentEco = eco
      localStorage.setItem('ecosystem', eco)
    }
  }
})