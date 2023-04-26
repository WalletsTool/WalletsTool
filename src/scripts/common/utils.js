export const utils = {
    sleep(delay) {
        let number = (Math.random() * (Number(delay[1]) - Number(delay[0])) + Number(delay[0])).toFixed(3);
        console.log('delay:', number, 's')
        return new Promise((resolve) => setTimeout(resolve, number * 1000));
    },
}
