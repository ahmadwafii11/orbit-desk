interface IconProps {
    size?: number; // Sisi icon dalam piksel
    className?: string;
}

// Logo Orbit Desk.
// Menggunakan versi dari icon asli (3264 px)
export function Icon({ size = 29 }: IconProps) {
    return (
        <>
            <img 
                className="app-icon"
                src="/orbit_desk_hd3.png" 
                width={size}
                height={size}
                alt="Orbit Desk" 
                draggable={false}
            />
        </>
    );
}