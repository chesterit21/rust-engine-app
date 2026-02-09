
import axios from 'axios';

const API_BASE_URL = 'http://localhost:3000/api';

export const api = axios.create({
  baseURL: API_BASE_URL,
});

export const endpoints = {
  projects: '/projects',
  modules: '/modules',
  userStories: '/user-stories',
  useCases: '/use-cases',
  // Add others as needed
};
