document.addEventListener('DOMContentLoaded', () => {
    const translations = {
        fr: {
            feedBtn: "Nourrir",
            playBtn: "Jouer",
            alreadyFed: "Déjà nourri",
            alreadyPlayed: "Déjà joué",
            fedCount: (n) => `Nourri ${n} fois aujourd'hui`,
            playedCount: (n) => `A joué ${n} fois aujourd'hui`,
            loading: "Chargement...",
            errorSync: "Erreur de connexion",
            errorAction: "Erreur lors de l'action"
        },
        en: {
            feedBtn: "Feed",
            playBtn: "Play",
            alreadyFed: "Already fed",
            alreadyPlayed: "Already played",
            fedCount: (n) => `Fed ${n} times today`,
            playedCount: (n) => `Played ${n} times today`,
            loading: "Loading...",
            errorSync: "Connection error",
            errorAction: "Action error"
        },
        de: {
            feedBtn: "Füttern",
            playBtn: "Spielen",
            alreadyFed: "Schon gefüttert",
            alreadyPlayed: "Schon gespielt",
            fedCount: (n) => `${n} Mal gefüttert heute`,
            playedCount: (n) => `${n} Mal gespielt heute`,
            loading: "Laden...",
            errorSync: "Verbindungsfehler",
            errorAction: "Aktionsfehler"
        }
    };

    let currentLang = localStorage.getItem('lang') || 'fr';

    const elements = {
        moodText: document.getElementById('mood-text'),
        feedsCount: document.getElementById('feeds-count'),
        feedMessage: document.getElementById('feed-message'),
        feedBtn: document.getElementById('feed-btn'),
        playSection: document.getElementById('play-section'),
        playfulnessText: document.getElementById('playfulness-text'),
        playsCount: document.getElementById('plays-count'),
        playMessage: document.getElementById('play-message'),
        playBtn: document.getElementById('play-btn'),
        foxPlaceholder: document.querySelector('.fox-placeholder'),
        langSelect: document.getElementById('lang-select')
    };

    elements.langSelect.value = currentLang;
    elements.langSelect.addEventListener('change', (e) => {
        currentLang = e.target.value;
        localStorage.setItem('lang', currentLang);
        fetchStatus();
    });

    const getApiPath = (endpoint) => {
        const basePath = window.location.pathname.replace(/\/$/, '');
        return `${basePath}/api/${endpoint}?lang=${currentLang}`;
    };

    const updateUI = (data) => {
        const t = translations[currentLang];
        elements.moodText.textContent = data.mood_text;
        elements.feedsCount.textContent = t.fedCount(data.feeds_today);

        if (data.has_fed_today) {
            elements.feedBtn.disabled = true;
            elements.feedBtn.textContent = t.alreadyFed;
        } else {
            elements.feedBtn.disabled = false;
            elements.feedBtn.textContent = t.feedBtn;
        }

        if (data.level_id === 10) {
            elements.playSection.classList.remove('hidden');
            elements.foxPlaceholder.classList.add('rolling');
            elements.playfulnessText.textContent = data.playfulness_text;
            elements.playsCount.textContent = t.playedCount(data.plays_today);

            if (data.player_plays_today >= 3) {
                elements.playBtn.disabled = true;
                elements.playBtn.textContent = t.alreadyPlayed;
            } else {
                elements.playBtn.disabled = false;
                elements.playBtn.textContent = t.playBtn;
            }
        } else {
            elements.playSection.classList.add('hidden');
            elements.foxPlaceholder.classList.remove('rolling');
        }

        if (data.message) {
            elements.feedMessage.textContent = data.message;
        }
    };

    const fetchStatus = async () => {
        const t = translations[currentLang];
        try {
            const res = await fetch(getApiPath('state'));
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();
            updateUI(data);
        } catch (error) {
            console.error("Error fetching state:", error);
            elements.moodText.textContent = t.errorSync;
        }
    };

    const feedTamagotchi = async () => {
        if (elements.feedBtn.disabled) return;
        
        const t = translations[currentLang];
        elements.feedBtn.disabled = true;
        elements.feedBtn.textContent = t.loading;

        try {
            const res = await fetch(getApiPath('feed'), { method: 'POST' });
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();
            updateUI(data);
            fetchStatus();
        } catch (error) {
            console.error("Error feeding:", error);
            elements.feedMessage.textContent = t.errorAction;
            elements.feedBtn.disabled = false;
        }
    };

    const playWithTamagotchi = async () => {
        if (elements.playBtn.disabled) return;

        const t = translations[currentLang];
        elements.playBtn.disabled = true;
        elements.playBtn.textContent = t.loading;

        try {
            const res = await fetch(getApiPath('play'), { method: 'POST' });
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();

            elements.playfulnessText.textContent = data.playfulness_text;
            elements.playsCount.textContent = t.playedCount(data.plays_today);
            elements.playMessage.textContent = data.message;

            if (data.player_plays_today >= 3) {
                elements.playBtn.disabled = true;
                elements.playBtn.textContent = t.alreadyPlayed;
            } else {
                elements.playBtn.disabled = false;
                elements.playBtn.textContent = t.playBtn;
            }
        } catch (error) {
            console.error("Error playing:", error);
            elements.playMessage.textContent = t.errorAction;
            elements.playBtn.disabled = false;
        }
    };

    elements.feedBtn.addEventListener('click', feedTamagotchi);
    elements.playBtn.addEventListener('click', playWithTamagotchi);
    
    fetchStatus();
});
