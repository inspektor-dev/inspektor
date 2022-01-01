import axios from "axios";

let token = localStorage.getItem("access-token");
if (token != null) {
    axios.defaults.headers["Auth-Token"] = token;
}
axios.defaults.baseURL = "http://localhost:3123"
axios.interceptors.response.use(function (res) {
    return res.data
}, function (error) {
    console.log("error", error)
    Promise.reject(error)
})

export default {
    login: async (username, password) => {
        let res = await axios.post("/login", { username, password })
        return res.data.token
    },
    getDatasources: async() =>{
        let res = await axios.get("/datasource")
        return res.data
    }
}