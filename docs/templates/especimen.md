# Plantilla · Espécimen

> Copia este archivo para documentar una especie (flora o fauna). En este repo las fichas se
> **generan** con `cargo run -p andes-core --bin gen-species`; usa esta plantilla solo como
> referencia de campos.

---

## <Nombre común> — *<Nombre científico>*

- **Grupo:** flora / fauna
- **Endémica de Chile:** sí / no
- **Estado de conservación (UICN):** <LC/NT/VU/EN/CR/…>
- **Capa(s) / zona:** <Andes / Precordillera / Valle / Costa / Playa / Mar>
- **Actividad (fauna):** <día / noche / crepúsculo / continua>
- **Tipo en código:** `<tipo>`

### Descripción / porte

<Descripción fiel de la silueta/hábito, que es la especificación de dibujo: porte general,
proporciones, rasgos distintivos. En las fichas generadas sale del campo `porte` de
`flora_data.rs`/`fauna_data.rs`.

### Especificación de sprite

| Campo | Valor |
|-------|-------|
| Tamaño (px) | <ancho × alto> |
| Paleta | <caracteres → claves de `get_palette` o `#hex`> |
| Frames | <lista de poses: idle, caminar, volar…> |
| Frame rate | <fps> |
| Anclaje | <pies / centro> |
| Escala con parallax | <sí/no, capa> |

### Comportamiento

- Desplazamiento: <caminar / volar / nadar / estático>
- Spawn: <cada cuántos chunks, rareza, condiciones>
- Interacciones: <con agua, con otras especies, con el clima>

### Notas

- TODO: <pendientes>

### Referencia

- **Wikipedia (estable):** <https://es.wikipedia.org/wiki/Especie>
- **Foto PD/CC0 (forma):** <enlace a Commons/PhyloPic>

### Referencias

- <enlaces internos>
