import { mount } from 'svelte'
import './app.scss'
import 'bootstrap/dist/js/bootstrap.bundle.min.js'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app'),
})

export default app
