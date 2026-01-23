
import { FC, MouseEvent } from 'react';
import { redirectToLogin } from '@/services/api';
import './Landing.css';

export const Landing: FC = () => {
  const handleLogin = (e: MouseEvent) => {
    e.preventDefault();
    redirectToLogin();
  };

  return (
    <div className="landing-page">
      <div className="bg-animation"></div>

      {/* Navigation */}
      <nav className="landing-nav">
        <div className="nav-container">
          <div className="landing-logo">
            <i className="fas fa-cube"></i>
            <span>Enterprise App</span>
          </div>
          <ul className="landing-nav-links">
            <li><a href="#features">Features</a></li>
            <li><a href="#about">About</a></li>
            <li><a href="#contact">Contact</a></li>
          </ul>
          <button onClick={handleLogin} className="btn-login">
            <i className="fas fa-sign-in-alt"></i> Sign In
          </button>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="hero">
        <div className="hero-content">
          <h1>
            Enterprise Management
            <br />
            <span className="gradient-text">Platform</span>
          </h1>
          <p>
            Secure, scalable, and modern application powered by 
            enterprise-grade SSO authentication. Manage your entire 
            organization with confidence.
          </p>
          <div className="hero-buttons">
            <button onClick={handleLogin} className="btn-primary">
              Get Started
              <i className="fas fa-arrow-right"></i>
            </button>
            <a href="#features" className="btn-secondary">
              Learn More
            </a>
          </div>
        </div>
        <div className="hero-visual">
          <div className="dashboard-preview">
            <div className="preview-header">
              <div className="preview-dot red"></div>
              <div className="preview-dot yellow"></div>
              <div className="preview-dot green"></div>
            </div>
            <div className="preview-content">
              <div className="preview-bar"></div>
              <div className="preview-bar"></div>
              <div className="preview-bar"></div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="features" id="features">
        <h2>Powerful Features</h2>
        <div className="features-grid">
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-shield-halved"></i>
            </div>
            <h3>Secure Authentication</h3>
            <p>
              Enterprise-grade SSO with OAuth 2.0, JWT tokens, and 
              multi-factor authentication support.
            </p>
          </div>
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-users-gear"></i>
            </div>
            <h3>User Management</h3>
            <p>
              Complete RBAC system with granular permissions, 
              groups, and role-based access control.
            </p>
          </div>
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-building"></i>
            </div>
            <h3>Multi-Tenancy</h3>
            <p>
              Support for multiple organizations with isolated 
              data and customizable settings per tenant.
            </p>
          </div>
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-gauge-high"></i>
            </div>
            <h3>High Performance</h3>
            <p>
              Built with Rust for blazing-fast performance and 
              memory safety. Handle millions of requests.
            </p>
          </div>
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-mobile-screen"></i>
            </div>
            <h3>Responsive Design</h3>
            <p>
              Beautiful UI that works perfectly on desktop, 
              tablet, and mobile devices.
            </p>
          </div>
          <div className="feature-card">
            <div className="feature-icon">
              <i className="fas fa-chart-line"></i>
            </div>
            <h3>Analytics & Reports</h3>
            <p>
              Comprehensive analytics dashboard with real-time 
              insights and exportable reports.
            </p>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer>
        <p>&copy; 2026 Enterprise App. Powered by SecureAuth SSO. All rights reserved.</p>
      </footer>
    </div>
  );
};
