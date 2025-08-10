import { defineStore } from 'pinia'

export type EcosystemKey = 'eth' | 'sol'

export const useEcosystemStore = defineStore('ecosystem', {
  state: () => ({
    currentEco: (localStorage.getItem('ecosystem') as EcosystemKey) || 'eth' as EcosystemKey,
    supported: ['eth', 'sol'] as EcosystemKey[],
  }),
  actions: {
    setEco(eco: EcosystemKey) {
      if (!this.supported.includes(eco)) return
      this.currentEco = eco
      localStorage.setItem('ecosystem', eco)
    }
  }
})

