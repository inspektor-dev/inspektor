
import { createStore } from 'vuex'

import api from "@/api/api";

function getDefaultStore() {
    return {
        datasources: [],
        tempDatasource: [],
        users: [],
        isAdmin: false,
        sup: true,
        count: 1,
        config: {},
    }
}

const store = createStore({
    state() {
        return getDefaultStore()
    },
    mutations: {
        setDatasource(state, datasource) {
            if (state.datasources == undefined) {
                state.datasources = datasource
                return
            }
            while (state.datasources.length) {
                state.datasources.pop()
            }
            datasource.forEach(data => {
                state.datasources.push(data)
            })
        },
        setUsers(state, users) {
            state.users = users
        },
        setIsAdmin(state, isAdmin) {
            state.isAdmin = isAdmin
        },
        reset(state) {
            state = getDefaultStore()
        },
        increment(state) {
            state.count++
        },
        config(state, data) {
            state.config = data
        },
        setTempDatasource(state, datasource) {
            state.tempDatasource = datasource
        }
    },
    actions: {
        async init({ commit }) {
            api.intializeToken()
            let roles = await api.getRoles();
            if (roles.indexOf('admin') == -1) {
                console.log("ignoring")
                return
            }
            let config = await api.config()
            commit("config", config)
            commit('setIsAdmin', true)
        },
        async updateDatasource({ commit }) {
            console.log("datasource updated")
            let datasources = await api.getDatasources();
            let sessions = await api.getSessions();
            // merge session and meta in same object.
            for (let i = 0; i < datasources.length; i++) {
                for (let j = 0; j < sessions.length; j++) {
                    if (sessions[j].objectID == datasources[i].id && sessions[j].meta.expiresAt == 0) {
                        datasources[i].sessionMeta = sessions[j].meta
                    }
                }
            }
            commit("setDatasource", datasources)
            let tempSesions = await api.getTempCredentials()
            let tempDatasources = []
            for (let k= 0; k< tempSesions.length; k++){
                let datasource = tempSesions[k].datasource;
                datasource.sessionMeta = tempSesions[k]
                tempDatasources.push(datasource)
            }
            commit("setTempDatasource", tempDatasources)
        },
        async updateUsers({ commit }) {
            let users = await api.getUsers()
            console.log(users)
            commit("setUsers", users)
        },
        async reset({ commit }) {
            commit("reset")
        },
        async refreshConfig({ commit }) {
            let config = await api.config()
            commit("config", config)
        }
    },
    getters: {
        datasources(state) {
            return state.datasources
        }
    }
})



export default store;