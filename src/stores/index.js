import {defineStore} from 'pinia'
import {ref} from "vue"

export const confettiStore = defineStore('confetti', () => {
    const status = ref(true)

    function changeStatus(value) {
        status.value = value
    }

    return {status, changeStatus}
})