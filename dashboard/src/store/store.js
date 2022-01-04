
import { createStore } from 'vuex'

import api from "@/api/api";

function getDefaultStore() {
    return {
        datasources: [],
        users: [],
        isAdmin: false,
    }
}

const store = createStore({
    state() {
        return getDefaultStore()
    },
    mutations: {
        setDatasource(state, datasource) {
            state.datasources = datasource
        },
        setUsers(state, users) {
            state.users = users
        },
        setIsAdmin(state, isAdmin) {
            state.isAdmin = isAdmin
        },
        reset(state) {
            state = getDefaultStore()
        }
    },
    actions: {
        async init({ commit }) {
            let roles = await api.getRoles();
            if (roles.indexOf('admin') == -1) {
                console.log("ignoring")
                return
            }
            commit('setIsAdmin', true)
        },
        async updateDatasource({ commit }) {
            let datasources = await api.getDatasources();
            commit("setDatasource", datasources)
        },
        async updateUsers({ commit }) {
            let users = await api.getUsers()
            console.log(users)
            commit("setUsers", users)
        },
        async reset({commit}) {
            commit("reset")
        }
    }
})

export default store;